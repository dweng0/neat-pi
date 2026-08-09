// Neato D10 brain-transplant — ESP32 co-processor: STEP 2 — DRIVE A REAL WHEEL MOTOR
//
// Proves the control chain on real hardware: serial command -> ESP32 -> DRV8871
// H-bridge -> Neato 260-0016 wheel motor, with speed (PWM) and direction control.
// The DRV8871 (3.6 A peak, adj. current limit) CAN take the wheel's ~2.4 A stall,
// unlike the L293D used for the step-1 toolchain proof.
//
// Power: motor rail (12 V from the bench PSU) goes to the DRV8871 VM screw
// terminal, NOT through the ESP32. The ESP32 stays USB-powered. The DRV8871 GND
// MUST tie to ESP32 GND (common ground) or the IN1/IN2 logic floats.
//
// Serial protocol (115200 baud, newline-terminated):
//   F <0-255>   forward at PWM duty   (e.g. "F 180")
//   R <0-255>   reverse at PWM duty   (e.g. "R 200")
//   S           stop / coast
//   B           brake (short the motor)
//   (bare number, e.g. "150", is treated as forward at that duty)
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
//   GND    -> DRV8871 GND   (common ground with the PSU — REQUIRED)
//   GPIO2  = onboard LED heartbeat

#include <Arduino.h>

static const uint8_t LED_PIN = 2;
static const uint8_t PIN_IN1 = 26;   // DRV8871 IN1
static const uint8_t PIN_IN2 = 27;   // DRV8871 IN2

// ESP32 LEDC PWM: one channel per input, since the DRV8871 has no separate
// enable pin — PWM must ride on whichever input is the active-direction one.
static const int      PWM_CH_IN1 = 0;
static const int      PWM_CH_IN2 = 1;
static const int      PWM_FREQ   = 20000;  // 20 kHz — above audible, easy for the DRV8871
static const int      PWM_RES    = 8;      // 8-bit -> duty 0..255

static uint32_t lastBeat = 0;

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

void setup() {
  pinMode(LED_PIN, OUTPUT);
  ledcSetup(PWM_CH_IN1, PWM_FREQ, PWM_RES);
  ledcSetup(PWM_CH_IN2, PWM_FREQ, PWM_RES);
  ledcAttachPin(PIN_IN1, PWM_CH_IN1);
  ledcAttachPin(PIN_IN2, PWM_CH_IN2);
  motorStop();

  Serial.begin(115200);
  delay(200);
  Serial.println();
  Serial.println("[neato-esp32] STEP 2 DRV8871 motor driver online.");
  Serial.println("[neato-esp32] cmds: 'F 180' fwd, 'R 200' rev, 'S' stop, 'B' brake.");
}

void handleCommand(String line) {
  line.trim();
  if (line.length() == 0) return;

  char c = toupper(line.charAt(0));

  if (c == 'S') { motorStop();  Serial.println("[motor] stop");  return; }
  if (c == 'B') { motorBrake(); Serial.println("[motor] brake"); return; }

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

  while (Serial.available() > 0) {
    handleCommand(Serial.readStringUntil('\n'));
  }
}
