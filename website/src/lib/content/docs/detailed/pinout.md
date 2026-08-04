# Connector Pinout

> **Note:** A more detailed version of this specification is coming soon.

For now use the KiCAD footprints, symbols and the [`GpioBank`](https://docs.rs/xpanse-api/latest/xpanse_api/gpio_bank/struct.GpioBank.html) type as reference.

Also here is a table, it is missing PWM

| PIN NR | FUNCTION1 | FUNCTION2 |
| ------ | --------- | --------- |
| 1      | GND       |           |
| 2      | 3V3       |           |
| 3      | MD0       |           |
| 4      | MD1       |           |
| 5      | GPIO0     | I2C_SCL   |
| 6      | GPIO1     | I2C_SDA   |
| 7      | GPIO2     | SPI_SCK   |
| 8      | GPIO3     | SPI_MISO  |
| 9      | GPIO4     | SPI_MOSI  |
| 10     | GPIO5     | UART_TX   |
| 11     | GPIO6     | UART_RX   |
| 12     | GPIO7     | ADC0      |
| 13     | GPIO8     | ADC1      |
| 14     | GPIO9     |           |
