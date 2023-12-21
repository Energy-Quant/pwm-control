use channel_reader::{CaptureTimer, ChannelReader};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

const VESC_PIN_1:i8 = 6;
const VESC_PIN_2:i8 = 7;
const X_INPUT:i8 = 4;
const Y_INPUT:i8 = 5;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    loop {
        readings = reading();
        calculate_speed = calculate(readings[0], readings[1]);
        control(calculate_speed[0], calculate_speed[1]);
    
        FreeRtos::delay_ms(12);    
    }
}

fn control(left_speed:u32, right_speed:u32) {

    // Take Peripherals
    let peripherals = Peripherals::take().unwrap();

    // Configure and Initialize LEDC Timer Driver
    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default()
            .frequency(50.Hz())
            .resolution(Resolution::Bits14),
    ).unwrap();

    // Configure and Initialize LEDC Driver
    let mut driver_1 = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver,
        peripherals.pins.VESC_PIN_1,
    )
    .unwrap();
      // Configure and Initialize LEDC Driver
    let mut driver_2 = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver,
        peripherals.pins.VESC_PIN_2,
    )
    .unwrap();

    // Get Max Duty and Calculate Upper and Lower Limits for Servo
    let max_duty_1 = driver_1.get_max_duty();
    let min_limit_1 = max_duty_1 * 25 / 1000;
    let max_limit_1 = max_duty_1 * 125 / 1000;
    
    let max_duty_2 = driver_2.get_max_duty();
    let min_limit_2 = max_duty_2 * 25 / 1000;
    let max_limit_2 = max_duty_2 * 125 / 1000;

    // Define Starting Position
    driver_1
        .set_duty(map(left_speed, 0, 180, min_limit, max_limit))
        .unwrap();
    driver_2
        .set_duty(map(right_speed, 0, 180, min_limit, max_limit))
        .unwrap();

}


// Function that maps one range to another
fn map(x: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> u32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}




fn reading() -> [i64; 2] {
    let capture_timer = CaptureTimer::new(0).unwrap();

    let channel1 = ChannelReader::new(&capture_timer, X_INPUT).unwrap();
    let channel2 = ChannelReader::new(&capture_timer, Y_INPUT).unwrap();
    let readings:[i64; 2] = [channel1.get_value(), channel2.get_value()];
    return readings;
}


fn calculate(x:i32 , y:i32) -> [u32; 2] {

    let mut left:f32 = 0.0;
    let mut right:f32 = 0.0;
    let mut angle_rad:f32 = 0.0; //угол в радианах
    let mut quarter:i8 = 0; //четверть
    let mut angle:f32 = 0.0; //угол в градусах
    let mut sin_angle:f32 = 0.0; // синус угла

    if x != 0 {
    
        //проверка четверти
        if x > 0 && y > 0 {
            quarter = 1;
            angle_rad = (y/x).atan();

        } else if x < 0 && y > 0 {
            quarter = 2;
            angle_rad = (y/x).atan();

        } else if x < 0 && y < 0 {
            quarter = 3;
            angle_rad = (y/x).atan();

        } else {
            quarter = 4;
            angle_rad = (y/x).atan();
        }


    } else {
        angle_rad = 90.to_radians();
    }

    //нахождение синуса угла
    sin_angle = angle_rad.sin();

//left and right wheel
    if quarter == 1 || quarter == 4 {
        left = y; 
        right = y * sin_angle;

    } else if quarter == 2 || quarter == 3 {
        left = y * sin_angle; 
        right = y; 
    }

    let mut speed:[u32; 2] = [left, right];
    return speed;
}