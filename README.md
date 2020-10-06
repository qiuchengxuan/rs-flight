rs-flight
=========

Flight controller written in Rust, working in progress

Progress
========

* Implemented
  - [x] USB console serial
  - [x] Barometer/Gyroscope/Accelerometer
  - [x] IMU
  - [x] DMA based OSD/HUD
  - [x] SDCARD read & write
  - [x] Logger
  - [x] YAML based config file
  - [x] Static memory allocator
  - [x] Battery Voltage ADC
  - [x] PWM & ESC
  - [x] SBUS Receiver
  - [x] GNSS UBX Protocol
  - [x] Displacement integral
  - [x] CRC based OSD font check
  - [x] timer based task scheduler
  - [x] software interrupt based event
  - [x] GNSS improved AHRS
  - [x] Complementary filter
  - [x] GNSS NMEA Protocol
  - [x] QMC5883L magnetometer
* WIP
  - [ ] Stabilizer
  - [ ] DMA based I2C
* Future
  - [ ] INS calibration
  - [ ] Replace JSON with Mavlink
  - [ ] Setup calibration
  - [ ] Blackbox
  - [ ] DMA based SDCARD read & write
  - [ ] Navigation
  - [ ] EKF filter
  - [ ] Involve async syntax
  - [ ] Camera distortion adaption

Memory allocator
================

Fow now it's using alloc-cortex-m as memory allocator,
but there's some alternatives to choose, e.g.

* linked_list_allocator
* static-alloc

data-flow
=========

* Speedometer

  ```plantuml
  @startuml
  ditaa

  +-----------+ Pressure +-----------+       Derivative
  | Barometer |--------->| Altimeter |-----------------+
  +-----------+          +-----------+                 |
                                                       v
  +---------------+ Accel  +-------+            +-------------+
  | Accelerometer |------->|       |            |             |
  +---------------+        |       | Integral   |             |
                           |  IMU  |----------->| Speedometer |
  +-----------+     Gyro   |       |            |             |
  | Gyroscope |----------->|       |            |             |
  +-----------+            +-------+            +-------------+
                                                       ^
  +-----------+                           Velocity     |
  | GNSS      |----------------------------------------+
  +-----------+
  ```
  
* SINS

  ```plantuml
  @startuml
  ditaa

  +-----------+ Pressure +-----------+       Altitude
  | Barometer |--------->| Altimeter |-------------+
  +-----------+          +-----------+             |
                                                   v
  +---------------+ Accel  +-------+            +------+
  | Accelerometer |------->|       |            |      |
  +---------------+        |       | Integral   |      |
                           |  IMU  |-------+--->| SINS |
  +-----------+     Gyro   |       |       |    |      |
  | Gyroscope |----------->|       |       |    |      |
  +-----------+            +-------+       |    +------+
                                           |       ^
                        +-------------+    |       |
                        | Speedometer |----+       |
                        +-------------+            |
  +-----------+  Position                          |
  | GNSS      |------------------------------------+
  +-----------+
  ```
  
* Output

  ```plantuml
  @startuml
  ditaa
  +-------------+         +-----+
  | Speedometer |-------->|     |  AoA
  +-------------+         | AoA |----------------+
                     +--->|     |                |
                     |    +-----+                v
  +-----+ Attitude   |                     +-----------+
  | IMU |------------+-------------------->| Stablizer |-------+
  +-----+                                  +-----------+       |
                                                               |
  +------+ Steerpoint   +-----------+                          |
  | SINS |------------->| Autopilot |---------------+          |
  +------+              +-----------+               |          |      +-----------+    +----------+
                                                    |          +----->|           |    |          |
  +----------+ Remote controller +---------------+  +---------------->|   Mixer   |--->|   PWMs   |
  | Receiver |------------------>| Configuration |------------------->|           |    |          |
  +----------+                   +---------------+                    +-----------+    +----------+
  ```

* Telemetry

  ```plantuml
  @startuml
  ditaa
  
  +-------------+       Speed vector                            +----------+
  | Speedometer |----+------------------------+             +-->| Blackbox |
  +-------------+    |                        |             |   +----------+
                     |     +-----+            v             | 
                     +---->|     |  AoA   +-----------+     |
                           | AoA |------->| Telemetry |-----+
                     +---->|     |        +-----------+     |
                     |     +-----+          ^ ^             |
  +-----+            |                      | |             |
  | IMU |------------+----------------------+ |             | 
  +-----+    Attitude                         |             | 
                                              |             | 
  +------+   Postion & Steerpoint             |             |   +----------+
  | SINS |------------------------------------+             +-->| HUD(OSD) |
  +------+                                                      +----------+
  @enduml
  ```
