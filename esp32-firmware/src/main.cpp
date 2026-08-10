// Neato D10 brain-transplant — ESP32 co-processor: STEP 2 — DRIVE + ENCODER
//
// Proves the control chain on real hardware: serial command -> ESP32 -> DRV8871
// H-bridge -> Neato 260-0016 wheel motor, with speed (PWM) and direction control.
// PLUS reads the wheel's magnetic quadrature encoder (A/B) so a spin turns into a
// counted, direction-aware position — the whole point of build step 2 (odometry).
//
// Power: motor rail (12 V from the bench PSU) goes to the DRV8871 VM screw
// terminal, NOT through the ESP32. The ESP32 stays USB-powered. The DRV8871 GND
// MUST tie to ESP32 GND (common ground) or the IN1/IN2 logic floats.
//
// Encoder: the LEGO 915-1055 board's A/B outputs are open-drain, idling high
// through an internal ~2.4 kOhm pull-up to a 5 V supply (>=4.5 V required). The
// ESP32 GPIOs are NOT 5 V tolerant, so each output is dropped to ~3 V by a single
// resistor (~3.3 kOhm) to GND, which forms a divider WITH that internal 2.4 kOhm
// pull-up. Blue = A -> GPIO32, Yellow = B -> GPIO33.
//
// Serial protocol (115200 baud, newline-terminated):
//   F <0-255>   forward at PWM duty   (e.g. "F 180")
//   R <0-255>   reverse at PWM duty   (e.g. "R 200")
//   S           stop / coast
//   B           brake (short the motor)
//   E           print encoder counts once
//   Z           zero the encoder counts
//   (bare number, e.g. "150", is treated as forward at that duty)
//
// While anything is moving, the loop also STREAMS encoder counts (~2/s) whenever
// they change — so a healthy sensor prints rising numbers during a spin, and a
// dead one stays silent.
//
// DRV8871 truth table (no EN pin — PWM rides directly on one input):
//   IN1     IN2     result
//   PWM     LOW     forward at duty
//   LOW     PWM     reverse at duty
//   LOW     LOW     coast (stop)
//   HIGH    HIGH    brake (both outputs low, motor shorted)
//
// Wiring:
//   GPIO26 -> DRV8871 IN1
//   GPIO27 -> DRV8871 IN2
//   GPIO32 <- encoder A (blue,  via ~3.3k-to-GND divider off the 5 V output)
//   GPIO33 <- encoder B (yellow, via ~3.3k-to-GND divider off the 5 V output)
//   GND    -> DRV8871 GND + encoder GND (orange) + divider GND  (common — REQUIRED)
//   GPIO2  = onboard LED heartbeat

#include <Arduino.h>

static const uint8_t LED_PIN = 2;
static const uint8_t PIN_IN1 = 26;   // DRV8871 IN1
static const uint8_t PIN_IN2 = 27;   // DRV8871 IN2
static const uint8_t PIN_ENC_A = 32; // encoder A (blue)
static const uint8_t PIN_ENC_B = 33; // encoder B (yellow)

// ESP32 LEDC PWM: one channel per input, since the DRV8871 has no separate
// enable pin — PWM must ride on whichever input is the active-direction one.
static const int      PWM_CH_IN1 = 0;
static const int      PWM_CH_IN2 = 1;
static const int      PWM_FREQ   = 20000;  // 20 kHz — above audible, easy for the DRV8871
static const int      PWM_RES    = 8;      // 8-bit -> duty 0..255

static uint32_t lastBeat = 0;

// ---- Encoder state (touched from ISRs) ----
volatile uint32_t edgeCountA = 0;    // raw transitions on A
volatile uint32_t edgeCountB = 0;    // raw transitions on B
volatile int32_t  encPos     = 0;    // signed quadrature position (4 counts / full A/B cycle)
volatile uint8_t  encPrev    = 0;    // last 2-bit AB state

// Standard quadrature transition table, indexed by (prev<<2 | now).
// +1 = one way, -1 = the other, 0 = no change / invalid double-step.
static const int8_t QDEC[16] = {
   0, -1, +1,  0,
  +1,  0,  0, -1,
  -1,  0,  0, +1,
   0, +1, -1,  0
};

// ---- Glitch filter (noise experiment — see hall-probe/ENCODER-DIAGNOSIS.md) ----
// Motor PWM chatters the encoder pins with a sub-µs dip roughly every 20 kHz
// PWM cycle (only ~0.4 V of noise margin on the divider). A minimum-SPACING
// filter was tried first and only downsampled the chatter (the noise repeats
// continuously, so an edge is always waiting when the window expires). This is
// a STABILITY filter instead: a new AB state is accepted only if it is still
// present glitchUs later — a PWM dip bounces back before confirmation, a real
// hall transition stays. 'G <us>' sets the window; G 0 (boot default) = off,
// identical to the original behavior. Keep the window small (1-10 µs): the ISR
// busy-waits for it on every candidate edge.
volatile uint32_t glitchUs      = 0;  // stability window; 0 = filter off
volatile uint32_t suppressedGl  = 0;  // candidate edges that didn't survive it

void IRAM_ATTR onEncEdge() {
  uint8_t a = (uint8_t)digitalRead(PIN_ENC_A);
  uint8_t b = (uint8_t)digitalRead(PIN_ENC_B);
  uint8_t now = (uint8_t)((a << 1) | b);
  uint8_t prev = encPrev;
  if (now == prev) return;             // interrupt fired but no net state change
  if (glitchUs) {
    uint32_t t0 = micros();
    while (micros() - t0 < glitchUs) {}          // hold...
    a = (uint8_t)digitalRead(PIN_ENC_A);         // ...and re-read
    b = (uint8_t)digitalRead(PIN_ENC_B);
    if ((uint8_t)((a << 1) | b) != now) { suppressedGl++; return; }
  }
  if (((prev >> 1) & 1) != a) edgeCountA++;
  if ((prev & 1) != b)        edgeCountB++;
  encPos += QDEC[(prev << 2) | now];
  encPrev = now;
}

void motorStop() {                       // coast: both inputs low
  ledcWrite(PWM_CH_IN1, 0);
  ledcWrite(PWM_CH_IN2, 0);
}

void motorBrake() {                      // active brake: both inputs high
  ledcWrite(PWM_CH_IN1, 255);
  ledcWrite(PWM_CH_IN2, 255);
}

void motorDrive(bool forward, int duty) {
  duty = constrain(duty, 0, 255);
  if (forward) {                         // PWM on IN1, IN2 low
    ledcWrite(PWM_CH_IN1, duty);
    ledcWrite(PWM_CH_IN2, 0);
  } else {                               // PWM on IN2, IN1 low
    ledcWrite(PWM_CH_IN1, 0);
    ledcWrite(PWM_CH_IN2, duty);
  }
}

void encoderSnapshot(uint32_t &a, uint32_t &b, int32_t &p) {
  noInterrupts();
  a = edgeCountA; b = edgeCountB; p = encPos;
  interrupts();
}

void encoderZero() {
  noInterrupts();
  edgeCountA = 0; edgeCountB = 0; encPos = 0; suppressedGl = 0;
  interrupts();
}

void setup() {
  pinMode(LED_PIN, OUTPUT);
  ledcSetup(PWM_CH_IN1, PWM_FREQ, PWM_RES);
  ledcSetup(PWM_CH_IN2, PWM_FREQ, PWM_RES);
  ledcAttachPin(PIN_IN1, PWM_CH_IN1);
  ledcAttachPin(PIN_IN2, PWM_CH_IN2);
  motorStop();

  // Encoder inputs — no internal pull (the external ~3.3k-to-GND divider sets the level).
  pinMode(PIN_ENC_A, INPUT);
  pinMode(PIN_ENC_B, INPUT);
  encPrev = (uint8_t)((digitalRead(PIN_ENC_A) << 1) | digitalRead(PIN_ENC_B));
  attachInterrupt(digitalPinToInterrupt(PIN_ENC_A), onEncEdge, CHANGE);
  attachInterrupt(digitalPinToInterrupt(PIN_ENC_B), onEncEdge, CHANGE);

  Serial.begin(115200);
  delay(200);
  Serial.println();
  Serial.println("[neato-esp32] STEP 2 DRV8871 motor driver + encoder online.");
  Serial.println("[neato-esp32] cmds: 'F 180' fwd, 'R 200' rev, 'S' stop, 'B' brake, 'E' enc, 'Z' zero, 'G 50' glitch-filter us.");
  uint32_t a, b; int32_t p;
  encoderSnapshot(a, b, p);
  Serial.printf("[enc] idle A=%u B=%u pos=%d (levels: A=%d B=%d)\n",
                a, b, p, digitalRead(PIN_ENC_A), digitalRead(PIN_ENC_B));
}

void handleCommand(String line) {
  line.trim();
  if (line.length() == 0) return;

  char c = toupper(line.charAt(0));

  if (c == 'S') { motorStop();  Serial.println("[motor] stop");  return; }
  if (c == 'B') { motorBrake(); Serial.println("[motor] brake"); return; }
  if (c == 'Z') { encoderZero(); Serial.println("[enc] zeroed");  return; }
  if (c == 'G') {                       // glitch filter: 'G 50' = 50 µs, 'G 0' = off
    glitchUs = (uint32_t)line.substring(1).toInt();
    suppressedGl = 0;
    Serial.printf("[enc] glitch filter = %u us\n", glitchUs);
    return;
  }
  if (c == 'E') {
    uint32_t a, b; int32_t p;
    encoderSnapshot(a, b, p);
    Serial.printf("[enc] A=%u B=%u pos=%d (levels: A=%d B=%d) filt=%uus supp=%u\n",
                  a, b, p, digitalRead(PIN_ENC_A), digitalRead(PIN_ENC_B),
                  glitchUs, suppressedGl);
    return;
  }

  // Pull a number off the rest of the line (or the whole line if it's bare).
  int duty;
  if (c == 'F' || c == 'R') {
    duty = line.substring(1).toInt();
  } else {
    duty = line.toInt();     // bare number -> forward
    c = 'F';
  }

  bool forward = (c == 'F');
  motorDrive(forward, duty);
  Serial.printf("[motor] %s duty=%d\n", forward ? "fwd" : "rev",
                constrain(duty, 0, 255));
}

void loop() {
  uint32_t now = millis();
  if (now - lastBeat >= 1000) {          // heartbeat
    lastBeat = now;
    digitalWrite(LED_PIN, !digitalRead(LED_PIN));
  }

  // Stream encoder counts whenever they change (so a spin prints rising numbers).
  static uint32_t lastEncPrint = 0;
  static uint32_t lastA = 0, lastB = 0;
  if (now - lastEncPrint >= 500) {
    lastEncPrint = now;
    uint32_t a, b; int32_t p;
    encoderSnapshot(a, b, p);
    if (a != lastA || b != lastB) {
      Serial.printf("[enc] A=%u B=%u pos=%d\n", a, b, p);
      lastA = a; lastB = b;
    }
  }

  while (Serial.available() > 0) {
    handleCommand(Serial.readStringUntil('\n'));
  }
}
