// Neato D10 brain-transplant — ESP32 co-processor: STEP 1 — DRIVE ONE MOTOR
//
// Proves the control chain: serial command -> ESP32 -> L293D H-bridge -> motor,
// with speed (PWM) and direction control. Use a SMALL bench motor (Elegoo kit),
// NOT a real Neato wheel motor — the L293D can't take the wheel's ~2 A stall.
// Real wheels come later on the DRV8871s.
//
// Serial protocol (115200 baud, newline-terminated):
//   F <0-255>   forward at PWM duty   (e.g. "F 180")
//   R <0-255>   reverse at PWM duty   (e.g. "R 200")
//   S           stop / coast
//   B           brake (short the motor)
//   (bare number, e.g. "150", is treated as forward at that duty)
//
// Wiring:
//   GPIO25 -> L293D pin 1  (EN1,2  = PWM speed)
//   GPIO26 -> L293D pin 2  (IN1    = direction)
//   GPIO27 -> L293D pin 7  (IN2    = direction)
//   GPIO2  = onboard LED heartbeat

#include <Arduino.h>

static const uint8_t LED_PIN = 2;
static const uint8_t PIN_EN  = 25;   // PWM enable  -> L293D EN1,2
static const uint8_t PIN_IN1 = 26;   // direction   -> L293D IN1
static const uint8_t PIN_IN2 = 27;   // direction   -> L293D IN2

// ESP32 LEDC PWM setup for the enable pin.
static const int      PWM_CH   = 0;
static const int      PWM_FREQ = 1000;   // 1 kHz — fine for a bench motor
static const int      PWM_RES  = 8;      // 8-bit -> duty 0..255

static uint32_t lastBeat = 0;

void motorStop() {                       // coast
  digitalWrite(PIN_IN1, LOW);
  digitalWrite(PIN_IN2, LOW);
  ledcWrite(PWM_CH, 0);
}

void motorBrake() {                      // active brake
  digitalWrite(PIN_IN1, HIGH);
  digitalWrite(PIN_IN2, HIGH);
  ledcWrite(PWM_CH, 255);
}

void motorDrive(bool forward, int duty) {
  duty = constrain(duty, 0, 255);
  digitalWrite(PIN_IN1, forward ? HIGH : LOW);
  digitalWrite(PIN_IN2, forward ? LOW  : HIGH);
  ledcWrite(PWM_CH, duty);
}

void setup() {
  pinMode(LED_PIN, OUTPUT);
  pinMode(PIN_IN1, OUTPUT);
  pinMode(PIN_IN2, OUTPUT);
  ledcSetup(PWM_CH, PWM_FREQ, PWM_RES);
  ledcAttachPin(PIN_EN, PWM_CH);
  motorStop();

  Serial.begin(115200);
  delay(200);
  Serial.println();
  Serial.println("[neato-esp32] STEP 1 motor driver online.");
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
