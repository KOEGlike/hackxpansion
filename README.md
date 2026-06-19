<h1 align="center">
    Hackxpansion
</h1>

<h3 align="center">
    <a href="https://hackxpansion.dino.icu">Website</a>
</h3>

<h4 align="center">
    A handheld console with 4 expansion solts
</h4>

<div align="center">
    <img src="blender/Hero.webp" width="600">
</div>

## Key Features

- Resistor based module idnetification at start up
- [2" 240x320 ST7789 LCD panel](https://www.buydisplay.com/2-inch-ips-tft-lcd-display-ips-panel-screen-240x320-for-smart-watch)
- RP2354 MCU
- Easily adaptable [module standard](https://hackxpansion.dino.icu/docs/detailed/card)
- Rust firmware with [embassy](https://embassy.dev/) and [slint-ui](https://slint.dev/)

## Why?

I really like the idea of modules, and modular things, and making people work together, so I made this.

## PCB

The console has two 4 layer PCBs designed in KiCAD, a main-board and a daughter-board, designed for JLC's basic process. 

### Main-Board

![main-board schematic](images/Main-Board-schematic.png)
![main-board pcb](images/Main-Board-pcb.png)
![main-board render](blender/MainBoardFront.webp)

### Daughter-Board

![daughter-board schematic](images/Daughter-Board-schematic.png)
![daughter-board pcb](images/Daughter-Board-pcb.png)
![daughter-board render](blender/DaughterBoardFront.webp)

## Making Your Own Modules

There are three simple steps to making your own module:

1. Design the module PCB in KiCAD
2. Design the module case in CAD
3. Write a driver for the module

There are detailed docs on the [hackxpansion website](https://hackxpansion.dino.icu/docs)

## Case

The case is designed in Autodesk Fusion, there are .f3z files in the hardware directory.

The models have parameters that can be adjusted to fit your needs.

## Shoutouts

Thank you so much for [Simon](https://github.com/NEOgHacking) who helped with the fanout of the RP, and also gave a ton of constructive feedback!!!!

## Zine

<img src="zine.png" alt="magazine page" width="800">

## BOM for Console

Also found in a [CSV file](hardware/hackxpansion/BOM.csv). Each component has an LCSC part number attached that can be used to find the component on the LCSC website.

|Item                      |Price per unit|Nr of units|Total price|Link                                                                                          |
|--------------------------|--------------|-----------|-----------|----------------------------------------------------------------------------------------------|
|PCB with PCBA             |$60.00        |1          |$60.00     |https://jlcpcb.com/                                                                           |
|Display                   |$4.36         |1          |$4.36      |https://www.buydisplay.com/2-inch-ips-tft-lcd-display-ips-panel-screen-240x320-for-smart-watch|
|2x7 2.54mm header         |$0.386        |10         |$3.860     |https://www.aliexpress.com/item/1005012166781874.html                                         |
|Battery 502035            |$6.64         |1          |$6.64      |https://www.aliexpress.com/item/1005008218024646.html                                         |
|M2 OD3 L2 heat set insert |$0.0696       |15         |$1.0440    |https://www.aliexpress.com/item/1005006798286851.html                                         |
|M2 L6 screw               |$0.0766       |5          |$0.3830    |https://www.aliexpress.com/item/4000970993800.html                                            |
|M2 L8 screw               |$0.0766       |2          |$0.1532    |https://www.aliexpress.com/item/4000970993800.html                                            |
|M2 L4 screw               |$0.0766       |8          |$0.6128    |https://www.aliexpress.com/item/4000970993800.html                                            |
|                          |              |Total:     |$77.05     |                                                                                              |
