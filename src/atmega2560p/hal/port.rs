//     RustDuino : A generic HAL implementation for Arduino Boards in Rust
//     Copyright (C) 2021  Prateek Kumar Gupta, Tulika Shukla, Sahil Aggarwal
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

//! Various pins and ports in the ATMEGA2560P chip is controlled here.
//! Section 13.2 to 13.4 of ATMEGA2560P datasheet.
//! https://ww1.microchip.com/downloads/en/devicedoc/atmel-2549-8-bit-avr-microcontroller-atmega640-1280-1281-2560-2561_datasheet.pdf
use core::{
    ptr::{read_volatile, write_volatile},
    usize,
};

/// Represents the name of the port , can vary from A-L leaving I.
#[derive(Clone, Copy)]
pub enum PortName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    J,
    K,
    L,
}

///     Each Port pin consists of 3 registers : PINx, DDRx, PORTx .
///     These registors control the ports.
///     Registers and bit references used here are written in general form.
///     A lower case “x” represents the numbering letter for the port, and a lower case “n” represents the bit number.
///     For example, PORTB3 for bit no. 3 in Port B, here documented generally as PORTxn
///     DDRx:*Data direction register*
///         The DDxn bit in the DDRx Register selects the direction of this pin.
///         DDxn = 1 => Pin x is configured as output.
///         DDxn = 0 => Pin x is configured as input.
///
///     PORTx: *Data register*
///     If PORTxn is written logic one when the pin is
///         configured as an input pin, the pull-up resistor is activated. To switch the
///         pull-up resistor off, PORTxn has to be written logic zero or the pin has to be
///         configured as an output pin. The port pins are tri-stated when reset condition
///         becomes active, even if no clocks are running.
///
///         PORTxn is written logic one when the pin is configured as an output pin,
///         the port pin is driven high (one). If PORTxn is written logic zero when the pin
///         is configured as an output pin, the port pin is driven low (zero).
///
///     PINx:*port input pins*
///         Writing a logic one to PINxn toggles the value of PORTxn, independent on the value of DDRxn.
pub struct Port {
    pin: u8,
    ddr: u8,
    port: u8,
}

/// The structure Pin contains the address of the port to which the pin belongs and the pin number
pub struct Pin {
    port: *mut Port,
    pin: usize,
}

/// Type 'IOMode'
/// Represents the Input/Output mode of the pin
pub enum IOMode {
    Input,
    Output,
}

impl Port {
    /// Returns mutable reference to the `Port` given `PortName`.
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        match name {
            PortName::A => &mut *(0x20 as *mut Port),
            PortName::B => &mut *(0x23 as *mut Port),
            PortName::C => &mut *(0x26 as *mut Port),
            PortName::D => &mut *(0x29 as *mut Port),
            PortName::E => &mut *(0x2C as *mut Port),
            PortName::F => &mut *(0x2F as *mut Port),
            PortName::G => &mut *(0x32 as *mut Port),
            PortName::H => &mut *(0x100 as *mut Port),
            PortName::J => &mut *(0x103 as *mut Port),
            PortName::K => &mut *(0x106 as *mut Port),
            PortName::L => &mut *(0x109 as *mut Port),
        }
    }

    ///Returns PortName of port of the given address input
    /// Panics if the addredd is invalid
    pub fn name(&self) -> PortName {
        let address = (self as *const Port) as usize; //get address of port
        match address {
            //return PortName based on the address read
            0x20 => PortName::A,
            0x23 => PortName::B,
            0x26 => PortName::C,
            0x29 => PortName::D,
            0x2C => PortName::E,
            0x2F => PortName::F,
            0x32 => PortName::G,
            0x100 => PortName::H,
            0x103 => PortName::J,
            0x106 => PortName::K,
            0x109 => PortName::L,
            _ => unreachable!(),
        }
    }

    /// Returns a `Some<Pin>` if pin number is valid and returns none if not valid
    pub fn pin(&mut self, pin: usize) -> Option<Pin> {
        if pin < 0x8 {
            Some(Pin { port: self, pin })
        } else {
            None
        }
    }
}

impl Pin {
    ///Return a pin for the given port name and pin number
    pub unsafe fn new(port: PortName, pin: usize) -> Option<Pin> {
        Port::new(port).pin(pin)
    }

    /// Change pin mode to input or output by changing the DDr register.
    /// If DDxn is written logic one, Pxn is configured
    ///as an output pin.
    /// If DDxn is written logic zero, Pxn is configured as an input pin.
    /// Section 13.2 of Atmega2605 datasheet
    pub fn set_pin_mode(&mut self, mode: IOMode) {
        //read the value of DDxn register
        let mut ddr_val = unsafe { read_volatile(&mut (*self.port).ddr) };

        //calculate the value to be written to DDxn register
        ddr_val &= !(0x1 << self.pin);
        ddr_val |= match mode {
            IOMode::Input => 0x0,
            IOMode::Output => 0x1 << self.pin,
        };

        // write the value to DDxn register
        unsafe { write_volatile(&mut (*self.port).ddr, ddr_val) }
    }

    ///Toggles the value of PORTxn by writing one to PINxn ,independent of the value of DDRxn.
    pub fn toggle(&mut self) {
        unsafe { write_volatile(&mut (*self.port).pin, 0x1 << self.pin) }
    }

    ///set the pin to high
    pub fn high(&mut self) {
        if self.pin >= 8 {
            return;
        } // Check if pin number is valid.
        unsafe {
            let p = read_volatile(&mut (*self.port).port); //reading the value of PORTxn.
            let ddr_value = read_volatile(&mut (*self.port).ddr); // Read the DDRxn register.
            if p == 0 && ddr_value == (0x1 << self.pin) {
                //toggling the value of PORTxn, if it isn't set to high.
                self.toggle();
            }
        }
    }

    ///set the pin to low
    pub fn low(&mut self) {
        // Check if pin number is valid.
        if self.pin >= 8 {
            return;
        }
        unsafe {
            let p = read_volatile(&mut (*self.port).port); //reading the value of PORTxn.
            let ddr_value = read_volatile(&mut (*self.port).ddr); // Read the DDRxn register.
            if p != 0 && ddr_value == (0x1 << self.pin) {
                //toggling the value of PORTxn, if it isn't set to low.
                self.toggle();
            }
        }
    }

    /// change pin mode to Output by changing the value of DDxn register to 1
    /// Section 13.2 of atmega2560 datasheet
    pub fn output(&mut self) {
        self.set_pin_mode(IOMode::Output);
    }

    /// change pin mode to Input by changing the value of DDxn register to 0
    /// Section 13.2 of atmega2560 datasheet
    pub fn input(&mut self) {
        self.set_pin_mode(IOMode::Input);
    }
}
