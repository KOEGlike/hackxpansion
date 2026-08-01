# Designing your first module

This guide will teach you how to create a simple module.

## Experience needed

To understand this guide, you should read the [Basics of Electronics](./basics-of-electronics) guide. If you don't understand something google is your best friend. If you have any further questions just ask in `#hackxpansion`

## Table of contents

:3

## Create a new project on the platform

Navigate to the projects tab in the sidebar and create a new project.

This is needed so you can reserve a pair of resistor values. This will come up later.

## Download software

- **KiCad:** This is the PCB editor program. Download the latest release. Can be downloaded from [here](https://www.kicad.org/download/)
- **Autodesk Fusion:** This is the 3D design program. You can get a student or hobbyist license. Can be downloaded from [here](https://www.autodesk.com/products/fusion-360/personal)
- **Git:** This is needed to publish your project and download resources. Can be downloaded from [here](https://git-scm.com/install/)

## Setup project

Download the empty base module from [here](https://download-directory.github.io/?url=https://github.com/KOEGlike/hackxpansion/tree/main/hardware/modules/empty). This will contain everything necessary for your first module of Basic or Advanced dificulty. It includes the librairies necessary to make a hackxpansion module and has the resistor detection already done.

## How does the console know which module you plugged in?

Each module has two resistors, which when connected each become the top resistor of a voltage divider, a 12bit ADC measures the resulting voltages, and loads the correct driver for that module. (In our case the bottom resistor is 10k)

![voltage divider](https://cdn.hackclub.com/019fbd38-4aff-77cf-9502-d0b30e614ee7/image.png)

When creating a project on the platform you'll get assigned these two resistor values that you need to put on your module.

## What connector do the modules use?

The modules connect with a standard right angle 2x7 2.54mm header, this way you don't even need to make a pcb to create new modules, just use a pref board, or you can just plug in dupont cables.

## What size should the modules be?

There is a standard size(check out the cad files), but you don't really need to follow it, the only thing you need to make sure is that it doesn't interfere with other modules, but if really don't want to follow it, you can just go wild.

## Creating a GitHub repo

You need this to publish project so everyone can see it. Teaching you Git and GitHub is not in the scope of this guide, but here is a good video.

Read this guide to know what to put int your repo

## Creating the PCB

There are two main parts to creating a circuit board, the first is creating the schematic, the secund is actually designing the PCB. But first you have to create a KiCAD project.

### Creating the KiCAD project

After cloning your repo, you should create new KiCAD project in the repo's folder called something like `pcb` or smth similar.

![create new project icon](https://cdn.hackclub.com/019fbdce-a866-7b94-a82b-d5fc920c8773/image.png)

### Creating the schematic

The schematic show how each component on your PCB should be connected, like to which pin of the module connector should a button be connected to.

To start editing the schematic of your KiCAD project just click on the `Schematic Editor icon`.

Here you can add **Symbols** with pressing `A`, that represent components on your PCB, they have all the pins that the "real" components have that will actually go on the circuit board, but they don't describe how the footprint will look on the PCB, this is because a button most of the time has 2 pins, so a common symbol can be used, but not the same size and shape, so multiple footprints are needed.

![schematic with the same 4 button symbols](https://cdn.hackclub.com/019fbdec-97b4-7472-8f79-a49d930a930b/image.png)

![pcb with 4 different buttons](https://cdn.hackclub.com/019fbdec-9993-7841-b994-56fe75f3ede0/image.png)

_The same button symbol used for 4 different buttons
(Buttons are called SW_Push in KiCAD)_
