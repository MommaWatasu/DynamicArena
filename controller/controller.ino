#include "MPU6050.h"
#include<MadgwickAHRS.h>
#include<BleGamepad.h>

MPU6050 mpu;
Madgwick MadgwickFilter;
BleGamepad bleGamepad("DynamicArena Controller", "MommaWatasu", 100);

const int MPU_addr=0x68;  // I2C address of the MPU-6050
int16_t ax, ay, az; // accel
int16_t gx, gy, gz; // gyro
float roll, pitch, yaw; // posture

void setup(){
  Wire.begin();
  mpu.initialize();
  Serial.begin(115200);
  delay(10);
  MadgwickFilter.begin(100);
  bleGamepad.begin();
}

void loop(){
  mpu.getMotion6(&ax, &ay, &az, &gx, &gy, &gz);
  MadgwickFilter.updateIMU(gx / 131.0, gy / 131.0, gz / 131.0, ax / 16384.0, ay / 16384.0, az / 16384.0);
  roll = MadgwickFilter.getRoll();
  pitch = MadgwickFilter.getPitch();
  yaw  = MadgwickFilter.getYaw();
  Serial.print(roll); Serial.print(",");
  Serial.print(pitch); Serial.print(",");
  Serial.print(yaw);
  Serial.print("\n");
  delay(10);

  if (bleGamepad.isConnected()) {
    //Serial.println("Send Accel Data");
    bleGamepad.setAccelerometer(roll, pitch, yaw);
    bleGamepad.sendReport();
  }
}