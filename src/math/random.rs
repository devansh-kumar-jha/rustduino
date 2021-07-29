//     RustDuino : A generic HAL implementation for Arduino Boards in Rust
//     Copyright (C) 2021  Devansh Kumar Jha, Indian Institute of Technology Kanpur
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU Affero General Public License as published
//     by the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU Affero General Public License for more details.
//
//     You should have received a copy of the GNU Affero General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>

use crate::delay::delay_ms;
use crate::hal::pin::Pins;

use crate::sensors::*;
use bit_field::BitField;

/// Selection of method to generate number.
#[derive(Clone, Copy)]
pub enum Generator {
    Analog,
    Mpu,
}

/// Controls the implementation of Random Number Generators.
/// # Elements
/// * `pins` - structure containing array to control all pins of micro-controller.
/// * `mpu` - a static mutable reference to the pointer location to control MPU6050 gyroscope.
/// * `mode` - a `Generator` object, which stores the implementation method for random number generator.
#[repr(C, packed)]
pub struct RandomNumberGenerator {
    pins: Pins,
    mpu: &'static mut MPU6050<'static>,
    mode: Generator,
}

impl RandomNumberGenerator {
    /// Create a new structure object for Random Number Generation.
    /// This structure contains elements for both ways of number generation implemented.
    /// # Returns
    /// * `a struct of type Random Number Generator` - to be used for the struct's implementation.
    pub fn new(mode1: Generator) -> RandomNumberGenerator {
        RandomNumberGenerator {
            pins: Pins::new(),
            mpu: MPU6050::new(),
            mode: mode1,
        }
    }

    /// Generation of random number through random noise in environment
    /// detected by read through analog pins input.
    /// # Returns
    /// * `a u8` - a random number generated by random noise as detected by analog pins during reading.
    pub fn generate_by_analog(&mut self) -> u8 {
        match self.mode {
            Generator::Mpu => unreachable!(),
            Generator::Analog => (),
        }

        let mut bits1: u8 = unsafe { xor_rotate() };

        bits1 = xor_shift(bits1);

        let bits2: u8 = unsafe { xor_rotate() };

        bits1 = xor(bits1, bits2);

        let mut lbuf: u8 = unsafe { xor_rotate() };
        let mut rbuf: u8 = unsafe { xor_rotate() };
        let buf: u8 = xor(lbuf, rbuf);

        let mut bits3: u8 = 0;

        for i in 1..4 {
            let left: u8;
            let right: u8;

            delay_ms(100);
            left = self.pins.analog[0].read() as u8;

            delay_ms(100);
            right = self.pins.analog[0].read() as u8;

            bits3 = xor(bits3, rotate(left, i));
            bits3 = xor(bits3, rotate(right, 7 - i));

            for j in 1..8 {
                let lb = left.get_bit(j);
                let rb = right.get_bit(j);

                if lb != rb {
                    if buf % 2 == 0 {
                        lbuf = push_left(lbuf, lb as u8);
                    } else {
                        rbuf = push_right(rbuf, lb as u8);
                    }
                }
            }
        }

        bits1 = xor_shift(bits1);

        bits1 = xor(bits1, bits3);

        xor(bits1, xor(lbuf, rbuf))
    }

    /// Generation of random number through random noise in environment
    /// detected through the MPU6050 sensor in the orthonormal set of axes.
    /// # Returns
    /// * `a u8` - a random number generated by multiple seeding within numbers generated by MPU6050 sensor.
    pub fn generate_by_mpu(&mut self) -> u8 {
        match self.mode {
            Generator::Analog => unreachable!(),
            Generator::Mpu => (),
        }

        let (a, b, c, d, e, f) = generate_mpu();

        let a1 = (a & 0x3) << 6;
        let a2 = (d & 0x3) << 6;
        let mut bits1 = xor(a1, xor(c << 4, xor(b << 2, xor(a, c >> 2))));
        let bits2 = xor(a2, xor(f << 4, xor(e << 2, xor(d, f >> 2))));

        bits1 = xor_shift(bits1);

        bits1 = xor(bits1, bits2);

        bits1
    }
}

/// Rotate the unsigned integer of 8 bits by n towards left
/// and surrounding back with the overflowing bits.
/// # Arguments
/// * `b` - a u8, the number whose bits will be rotated.
/// * `n` - a u8, by how many places bits are to be rotated.
/// # Returns
/// * `a u8` - the bit-shifted number.
pub fn rotate(b: u8, n: u8) -> u8 {
    (b >> n) | (b << (8 - n))
}

/// Get the bitwise XOR (exclusive OR) of two 8 bits unsigned integers.
/// # Arguments
/// * `a` - a u8, first unsigned integer.
/// * `b` - a u8, second unsigned integer.
/// # Returns
/// * `a u8` - bitwise XOR.
pub fn xor(a: u8, b: u8) -> u8 {
    (a | b) - (a & b)
}

/// XOR Shift for stability in number generation.
/// It implements one round of XORShift PRNG algorithm for statistical stability.
/// # Arguments
/// * `a` - a u8, the number whose bits will be shifted.
/// # Returns
/// * `a u8` - the bit-shifted number.
pub fn xor_shift(a: u8) -> u8 {
    let mut ans: u8 = 0;

    ans = xor(ans, a);
    ans = xor(ans, a >> 3);
    ans = xor(ans, a << 5);
    ans = xor(ans, a >> 4);

    ans
}

/// Generate XOR Rotation number.
/// # Returns
/// * `a u8` - A random number generated by various XOR's on sample generated through analog read.
pub unsafe fn xor_rotate() -> u8 {
    let mut bits1: u8 = 0;
    let mut obj = RandomNumberGenerator::new(Generator::Analog);

    for i in 1..8 {
        let a: u8 = obj.pins.analog[0].read() as u8;
        bits1 = xor(bits1, rotate(a, i));
        delay_ms(20);
    }

    bits1
}

/// Push the required bit with left bias.
/// # Arguments
/// * `val`    - a u8, the number to whom the bits are to be added.
/// * `change` - a u8, the extent of rotation before XOR of the bits of `val`.
/// # Returns
/// * `a u8` - The bit changed value of `val`.
pub fn push_left(val: u8, change: u8) -> u8 {
    xor(val << 1, xor(change, val))
}

/// Push the required bit with right bias.
/// # Arguments
/// * `val`    - a u8, the number to whom the bits are to be added.
/// * `change` - a u8, the extent of rotation before XOR of the bits of `val`.
/// # Returns
/// * `a u8` - The bit changed value of `val`.
pub fn push_right(val: u8, change: u8) -> u8 {
    xor(val >> 1, xor(change << 7, val))
}

/// Function to generate tuple containing u8 numbers
/// accordingly through MPU6050 Gyroscopic Sensor.
/// # Returns
/// * `a tuple of 6 u8's` - The x,y,z axes accelerations and gyroscopic detections by MPU6050 sensor respectively.
pub fn generate_mpu() -> (u8, u8, u8, u8, u8, u8) {
    let obj = RandomNumberGenerator::new(Generator::Mpu);

    obj.mpu
        .begin(MPUdpsT::MPU6050Scale250DPS, MPURangeT::MPU6050Range2G);

    obj.mpu.read_gyro();
    delay_ms(1000);

    obj.mpu.read_accel();
    delay_ms(1000);

    let d: u8 = obj.mpu.gyro_output[0] as u8;
    let e: u8 = obj.mpu.gyro_output[1] as u8;
    let f: u8 = obj.mpu.gyro_output[2] as u8;
    let a: u8 = obj.mpu.accel_output[0] as u8;
    let b: u8 = obj.mpu.accel_output[1] as u8;
    let c: u8 = obj.mpu.accel_output[2] as u8;
    (a, b, c, d, e, f)
}
