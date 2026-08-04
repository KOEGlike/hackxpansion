# Designing your first module

This guide will teach you how to get started creating a simple module. You can't copy this guide one to one, you have to put you own twist on it, and is not meant for copying.

## Experience needed

To understand this guide, you should read the [Basics of Electronics](./basics-of-electronics) guide. If you don't understand something, google is your best friend. If you have any further questions just ask in `#hackxpansion`

## Create a new project on the platform

Navigate to the projects tab in the sidebar and create a new project.

This is needed so you can reserve a pair of resistor values. This will come up later.

## Download software

- **KiCad:** This is the PCB editor program. Download the latest release. Can be downloaded from [here](https://www.kicad.org/download/)
- **Autodesk Fusion:** This is the 3D design program. You can get a student or hobbyist license. Can be downloaded from [here](https://www.autodesk.com/products/fusion-360/personal)
- **Git:** This is needed to publish your project and download resources. Can be downloaded from [here](https://git-scm.com/install/)

## Setup project

Download the Hackxpansion KiCAD library from [here](https://download-directory.github.io/?url=https://github.com/KOEGlike/hackxpansion/tree/main/hardware/kicad_lib). This will contain everything necessary for your first module of Basic or Advanced difficulty. It includes the libraries necessary to make a Hackxpansion module. Create a new folder and put it somewhere like `Documents` or in your repo, and move the downloaded library into it, so you can easily access it later.

## How does the console know which module you plugged in?

Each module has two resistors, which when connected each become the top resistor of a voltage divider. A 12bit ADC measures the resulting voltages, and loads the correct driver for that module. (In this case the bottom resistor is 10k)

![voltage divider](https://cdn.hackclub.com/019fbd38-4aff-77cf-9502-d0b30e614ee7/image.png)

When creating a project on the platform you'll get assigned these two resistor values that you need to put on your module.

## What connector do the modules use?

The modules uses a standard right angle 2x7 2.54mm header. This way you don't even need to make a pcb to create a new modules, just use a perfboard, or you can just plug in breadboard cables.

## What size should the modules be?

There is a standard size (check out the [cad files](https://szekelymikokollegium2.autodesk360.com/g/shares/SH28cd1QT2badd0ea72b21236d76e5e5ca90)). You don't really need to follow it, the only thing you need to make sure is that it doesn't interfere with other modules. But if you really don't want to follow it, you can just go wild.

## Creating a GitHub repo

You need this to publish project so everyone can see it. Teaching you Git and GitHub is not in the scope of this guide, but here is a good [guide](https://docs.github.com/en/get-started/start-your-journey/git-and-github-learning-resources).

Read this [guide](../shipping/design) to know what to put into your repo

## Creating the PCB

There are two main parts to creating a circuit board, the first is creating the schematic, the second is actually designing the PCB. But first you have to create a KiCAD project and import libraries.

### But wait, what's a PCB?

A `PCB` (Printed Circuit Board) is a board used to mechanically support and electrically connect electronic components. It is composed of layers of copper and a `dielectric material` (a material that acts as an "insulator" and isn't conductive). PCBs can have from 2-32 layers, although for this board, you will be working with 2 (maybe 4 if you need to).

In KiCad, if you look to the right, it shows the different layers:

![pcb layers](https://cdn.hackclub.com/019fc7cf-90ac-78d5-8c7b-9ff3b7f05e88/image.png)

`F.Cu` and `B.Cu` (Front Copper and Back Copper) are your copper layers (denoted by the `.Cu`). The only other layers that are important to us are `F/B.Silkscreen` and `Edge.Cuts`. The silkscreen layers allow us to put text/images and is usually that white text that you find on any PCB. `Edge.Cuts` is the layer for the edges of the board (e.g., where JLCPCB will cut to make the outline). For now we will focus on the copper layers and routing. You can add more copper layers later in the board settings.

Now you have to connect each of the components and `route` them (create copper lines between each of them).

_This section is from [here](https://blueprint.hackclub.com/starter-projects/flightcontroller)_

### Creating the KiCAD project

First you need to clone the repo you created on [GitHub](https://github.com). Then you need create a new KiCAD project in the repo's folder called something like `pcb` or smth similar.

![create new project icon](https://cdn.hackclub.com/019fbdce-a866-7b94-a82b-d5fc920c8773/image.png)

### Importing libraries

This allows you to use pre-made symbols and footprints. Click on `Preference` in the main KiCAD page

![KiCAD preferences](https://cdn.hackclub.com/019fc721-fdda-79e5-8c61-73eef75fe66d/image.png)

#### Symbol Library

First click on `Manage Symbol Libraries`. If you put the downloaded library folder in `Documents` or some sort of global place, click on `Global Libraries` then on the folder icon to locate the downloaded library. It should be named `Hackxpansion.kicad_sym`.

![Manage Global Symbol Libraries](https://cdn.hackclub.com/019fc727-48d0-7ced-be2c-61bf64b540fb/image.png)

If you put the downloaded library into your repo's folder, the process is identical to `Global Libraries`, but you should click on `Project Specific Libraries` instead.

![Manage Project Symbol Libraries](https://cdn.hackclub.com/019fc725-c76f-725b-bbfe-58af4ff54f2a/image.png)

#### Footprint library

Next you should click on `Manage Footprint Libraries`, the steps are the same as for the symbol library, but instead of selecting the `Hackxpansion.kicad_sym` file, you should select `Hackxpansion.pretty` folder.

### Creating the schematic

The schematic show how each component on your PCB should be connected, like to which pin of the module connector should a button be connected to.

To start editing the schematic of your KiCAD project just click on the `Schematic Editor icon`.

#### What is a symbol?

Here you can add **Symbols** with pressing `A`, that represent components on your PCB, they have all the pins that the "real" components have, that will actually go on the circuit board, but they don't describe how the footprint will look on the PCB, this is because a button most of the time has 2 pins, so a common symbol can be used, but buttons are not same size and shape, so multiple footprints are needed.

![schematic with the same 4 button symbols](https://cdn.hackclub.com/019fbdec-97b4-7472-8f79-a49d930a930b/image.png)

![pcb with 4 different buttons](https://cdn.hackclub.com/019fbdec-9993-7841-b994-56fe75f3ede0/image.png)

_The same button symbol used for 4 different buttons
(Buttons are called SW_Push in KiCAD)_

#### Adding module connector

Next you should add the symbol for the Hackxpansion male module connector, simply search `Hackxpansion` in the `Choose Symbol` menu, select the `Module_Male` symbol and add it to the schematic. (If it doesn't show up, you might need to restart KiCAD)

![Choose Symbol menu](https://cdn.hackclub.com/019fc733-e913-7392-815e-8437f6df6826/image.png)

#### Adding resistors

Each modules needs two resistors, so you need to add symbols for them, simply search `resistor` and choose the `R` symbol.

After adding one, simply select it and copy-paste it to make another one. You can move them by selecting them and pressing `M`.

![resistors and male module](https://cdn.hackclub.com/019fc741-0a80-7428-a382-5314fd52f6c7/image.png)

These resistors currently don't have any value attached to them, you can change this by double clicking on `R` or by clicking on the body of the resistor and pressing `E` and changing the `Value` field. Change one of the resistor's value your project's MD0 and the other to to MD1 (you can get these values from the project page of the website)

![](https://cdn.hackclub.com/019fc75d-be00-7cf2-bbad-9941d079b570/image.png)

Now you need to connect the bottom pin of the resistor with the value of `MD0` to the `MD0` pin on the `Module_Male` symbol, and the resistor with the value of `MD1` to the `MD1` pin on the symbol. You can simply click on the little circle on the bottom of the resistor and connect the wire to the symbol.

The top pin of the resistors should be connected to the `3V3` pin on the module symbol, which supplies power at 3.3 volts. This completes the voltage divider.

Here is how your schematic should be looking like:

![](https://cdn.hackclub.com/019fc75c-e397-73e3-9e86-f0022c9a62d2/image.png)

Now you got a module that can be detected by the console but doesn't do anything :yayayay:

#### Adding a button

Now lets add some actual features to your modules. Add a button by searching for `SW_Push` in the symbol selector menu. Connect one side of the button to the GND (ground/earth/negative of the battery/supply) pin on the module, and the other pin any GPIO (General-Purpose Input/Output) pin of the module symbol.

![added button](https://cdn.hackclub.com/019fc76c-5edd-7a16-a3c6-e0a60e330d96/image.png)

As you can see in the symbol of the button, it normally doesn't close the circuit, but when you push down on it, it connects the two points, and the microcontroller onboard the console can detect this change and do something, like move a player on the screen.

#### Adding power symbols and labels

For a module this simple your current setup is totally fine, but if your module has a bunch chips/components on it, connecting directly everything with these green wires becomes cumbersome, but there is a solution: labels and power symbols.

##### Power Symbols

Press `P` to open the power symbol selector menu and search for `+3V3` (yes there are negative voltages too) and place down the symbol somewhere, also search for `GND` and place that down somewhere.

Power always flows from top to bottom, so the arrow of the `+3v3` symbol should always point up, and the arrow of `GND` should always point downwards.

Next, disconnect the top part of the resistor from the `3V3` pin of the module symbol, by selecting one or multiple wires and pressing `Delete` on you keyboard. Now connect the pin of the `+3V3` symbol to the `3V3` pin of the module symbol. Copy-paste the `+3V3` power symbol and connect it to the two top pins of the resistor. Here is how it should look:

![added 3v3 symbol](https://cdn.hackclub.com/019fc777-fdcf-72b6-bafa-018d639a8fe6/image.png)

This may look like that the top pins of the resistors' is not connected to the `3V3` pin of the module symbol, but it is, if you click on a green wire that connects one of the `+3v3` symbols to something and press `` ` ``(the key left to `1`) it should highlight all the things that that wire connects

![highlighted cables](https://cdn.hackclub.com/019fc77e-6118-715e-be12-e7aec2136a3c/image.png)

Now do the same thing with `GND` and the right pin of the button:

![added gnd](https://cdn.hackclub.com/019fc785-2c22-7d92-8678-daec3953e742/image.png)

##### Labels

Labels are really similar to power symbols, the only difference is that labels are used for general signals and not power.

Create a new label by pressing `L` and give your label a name, in this case we will use this label to connect the button to the module symbol, so I'll name it `BTN1`

![BTN1 label](https://cdn.hackclub.com/019fc789-9b08-75c1-96ca-be5349ea6b2c/image.png)

Now delete the wire connecting the button an module symbol, connect the label to one of the pins of the module symbol, copy-paste the label and connect the new label the the left pin of the button symbol

(you can mirror the labels and symbols by selecting them and pressing `X` to horizontally mirror them and pressing `Y` to mirror them vertically)

If you also check this with `` ` `` then you can also see that `GPIO0` and the left pin of the the button are connected.

![connected labels](https://cdn.hackclub.com/019fc798-fb19-7547-857e-d505c39ba9ef/image.png)

#### Marking pins as unused

We only used `GPIO0`, `MD0`, `MD1`, `GND` and `3V3`, and didn't use a bunch of pins, you need to mark these as unused.

Click on the `No Connect Flag` icon on the right sidebar or press `Q`, and place these no connect symbols on all the pins that you didn't use

![NC flags](https://cdn.hackclub.com/019fc79e-94fa-73e7-85ef-e00d085f79fc/image.png)

#### Assigning footprint

As mentioned in [What is a symbol?](#what-is-a-symbol) a symbol can have multiple types of footprints assigned to it, so you need to specify which one you want to use.

Click on the `Assign Footprints` icon in the top bar:
![assign footprint button](https://cdn.hackclub.com/019fc7a3-a57a-77df-ab2d-a5b91dd95bb2/image.png)

This will bring up a table with all of your symbols, if the row is yellow, it means that there is no footprint assigned to that symbol and you need to assign one

![footprint assignment table](https://cdn.hackclub.com/019fc7a8-0872-7080-9f9e-d1d4c226f203/image.png)

There is search bar located in the top bar, here you can search for footprints. For the resistors I recommend you to choose 0805 footprints, simply search `R_0805` and choose the hand soldering variant by double clicking on it.

For the switch I would choose a 6mm push button, because it's really common, just search `sw_push_6mm`

#### (Bonus) Changing pin function on module symbol

The male module symbol is set up in a way if you right click on one of the pins, and click on `Pin Function`, here you can see what that pin is capable of, and change it's function to one of them

![pin functions](https://cdn.hackclub.com/019fc7be-46c9-7663-bfe4-7d1ce9507e6f/image.png)

In the module that you are designing you don't need this. Because it's really simple and don't use any protocols. But in complex modules this can be useful

### Creating the actual PCB

So far you have been only working on the schematic, but not the actual PCB, but the time has come

#### Importing schematic into PCB

Click the `Switch to PCB editor` button on the top bar in the schematic editor:

![Switch to PCB editor](https://cdn.hackclub.com/019fc7c3-5265-7dd5-b219-b3f4fe5521c1/image.png)

Then click the `Update PCB from Schematic` button in the top bar of the PCB editor to import your schematic into the PCB. You need to to this every time you make a change to the schematic

![Update PCB from Schematic](https://cdn.hackclub.com/019fc7c4-9738-7332-9c02-a21a3bcb2d1a/image.png)

Now all your components should show up:
![all components](https://cdn.hackclub.com/019fc7c6-6270-7ae8-8115-07af8ea8eb84/image.png)

The part of the module that is shaded in is the outline of the PCB, every component should go in there.

You may also notice some blue lines. These show the connections that you need to make.

#### Layout

Next you need lay out all of your pars, you can move them with `M` and rotate them with `R`, just like in the schematic editor.

Here is the layout I went with, but you could do anything:

![](https://cdn.hackclub.com/019fc7ca-17b9-787d-9bfb-bfda81039568/image.png)

#### Connecting components

Now that you have all your components laid out, it's time to connect them. Every component has several pads, they are red, because they are all on the front layer, if you click on one (that doesn't have an `x` on it, which mean it's NC) and press `X` you start **routing**, now a one/few pads will be highlighted that you need to connect to, simply click and route to the highlighted pads.

Here are some helpful shortcuts:

- `G` can be used to grab and push around traces
- If you want to delete a whole trace press `U` a bunch of times to select all the traces that section is connected to, and then press `Delete`

Here is mine:

![](https://cdn.hackclub.com/019fc7d7-4417-7eca-a5c1-99dbf7bb4f8a/image.png)

#### Adding a ground fill

You may have noticed that most of your PCB is empty, there's not a lot of copper on it. Your design will work, but it's bad for manufacturing and signal integrity. Watch [this](https://youtu.be/R3w4Go1s1hM?si=J2GRukJI-MTfpDRr) video if you want to learn more.

Add a Ground Fill by pressing the Draw Filled Zone button on the right sidebar, then click somewhere outside of your PCB.

![](https://cdn.hackclub.com/019fc7d7-c962-78d7-99ea-4b757f6e1b87/image.png)

Now select both copper layers, add a name, and select the `GND` net then press OK

![](https://cdn.hackclub.com/019fc7d8-6e6d-7fea-a3b6-8d76df661efe/image.png)

Now draw a rectangle around your PCB:

![](https://cdn.hackclub.com/019fc7d9-dd11-7c56-b977-31ad51e8a9a7/image.png)

Then press `B` to fill:

![](https://cdn.hackclub.com/019fc7da-6843-7ede-90f9-21e4de4476a0/image.png)

#### Adding stitching VIAs

VIAs are used to connect two or more different layers of a PCB. They work by drilling a small hole in the PCB, and filling that with copper.

You need VIAs to connect the ground pours on both of the layers, otherwise your PCB will have signal integrity problems (on a PCB as simple as this one, it will not be a problem, but it's best practice)

Click on the `Place VIA` button on the right sidebar

![](https://cdn.hackclub.com/019fc7da-6a4e-7b31-8477-356edb666cdd/image.png)

Then sprinkle some on your PCB then press `B` to fill again

![](https://cdn.hackclub.com/019fc7da-6c7a-7464-8744-3666a2020209/image.png)

#### Adding mounting holes

You will somehow need to attach your PCB 3D printed parts

Press `A` to add a new footprint, search for `m2` and select the M2 mounting hole.

Add two of this, and position them at two edges of the board, then press `B` to refill

![](https://cdn.hackclub.com/019fc859-0688-7d82-ad9a-67a01ca44ade/image.png)

You could also add mounting hole symbols in you schematic, assign footprints to them, and then you don't have to add them in the PCB editor, this is actually the recommended way.

#### Exporting a 3D model

Click the `3D Viewer` button in the top bar

![](https://cdn.hackclub.com/019fc85a-b378-74b5-9e43-5eca4da1faa5/image.png)

Here you can view your PCB in 3D.

You need to export it as a `.step` file, this is a file format that all CAD programs accept (like Autodesk Fusion), and is really easy to work with.

Go back to the PCB editor window, go to `File -> Export -> STEP/GLB/BREP`

![](https://cdn.hackclub.com/019fc860-4329-7b8f-909f-b7b3b02beca5/image.png)

Use these settings:

![](https://cdn.hackclub.com/019fc861-2197-7d87-8d6c-d728c11f3fb1/image.png)

And press export!

#### (Bonus) Routing and VIAs

On simple PCBs like this, you can do all of you routing on a single copper layer, but on more complex designs you need to use the back copper layer while routing.

When you entered routing mode by pressing `X` you can switch layers by pressing `V`, this will prompt you to place a VIA and got the the other layer

![](https://cdn.hackclub.com/019fc865-911b-7eb2-97d1-20ebdb8ece0e/image.png)

## Creating your case

This part of the guide will teach you how to create the case for your module

### Creating a Fusion project

Open Fusion and create a new folder called `My Module` or smth similar, upload the step file that you exported of your PCB and then create a new `Hybrid Design` in that folder.

Then drag your uploaded pcb model into the newly created hybrid design.

# Work In Progress

For now look at the [example](https://szekelymikokollegium2.autodesk360.com/g/shares/SH28cd1QT2badd0ea72b21236d76e5e5ca90)
