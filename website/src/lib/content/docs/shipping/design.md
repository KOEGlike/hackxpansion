# Shipping Your Design

Okay, you’ve designed your project digitally\! Congrats\! Before approving your projects, they need to meet our requirements.

There are two key areas of our Design Submission Requirements:

1. A good README
2. Fully completed design

**Missing any will get your project returned**. Read through all of them\! It’s easy to miss one.

## 1. A Good README

Your README is people’s first impression. Make it awesome\! Someone landing on your repository for the first time should understand:

- What your project is
- What it does
- Why it exists

If they have to open even a single file, your README is not doing its job. At minimum, your `README.md` file must include:

1\. Explanation of what your project is

- [x] Short description of what your project is\! Highlight what makes it unique
- [x] How do you use it? Be detailed\! Others can’t read your mind.
- [x] Why did you make it? Be personal\! Are you solving a problem? Trying to make something smaller than previously thought possible?

2\. Add images\! A picture is worth a thousand words. Include:

- [x] Screenshots of a full 3D model of your project fully assembled
- [x] Screenshots of your PCB with components
- [x] Anything else that makes it clear what your project is and what it’s for

3\. Add your MD0 and MD1 values for modules:

- [x] Link them to your public project page, like https://hackxpansion.koeg.dev/explore/1k:1k
- [x] Put it in a visible place

4\. Link resources that your app uses

- [x] Have a list of all the crates that export the resources that your app uses with links to all of them
- [x] Put it in a visible place

## 2. A fully finished design:

Whoa… A lot at first glance\! Breathe. You got this. It’s simpler than it seems\!

For a design to be 100% finished, someone else should be able to read your repo, understand, and replicate it… i.e. You need to include all files and instructions\!

A project that only you can make is not [**shipped**](/docs/requirements/what-is-shipping). It only lives in your head.

The design should also reasonably actually work\! Of course, you can’t be sure until building it, but stuff like floating parts, incomplete firmware, or parts attached with “magic” is a no-go.

#### At minimum, your project should be:

- [x] Original, custom design by you. **Not by AI, not a direct copy of a tutorial, or someone else.**
- [x] Has a complete CAD assembly, with all components (including electronics).
- [x] Have a concrete way to attach components (including electronics). Use screws, clips, etc. This is a product, not a demo, it should feel solid, and not held up by tape, glue, and dreams.
- [x] Someone else sanity checked your design\! It’s EASY to miss things. Ask a friend, in `#hackxpansion`, and fix them before submitting\!

#### Your GitHub repository needs to contain all your project files

- [x] A BOM (Bill of Materials) in CSV format, with links, and a line indicating the total cost\! Even if you own a part, still include it. Someone else needs to be able to build what you’ve designed
- [x] The source files of your PCB (`.kicad_pro`, `.kicad_sch`, `.kicad_pcb`, `.epro`, `gerbers.zip`, etc)
- [x] For 3D models, `.step` files of your project’s 3D CAD and the source design file (`.f3d`, `FCStd`, or a link to onshape)
- [x] Drivers for modules, and firmware for apps, make sure to include source code
- [x] ANY other files that are part of your project (libraries, references, etc.)
- [x] Make sure your repository is well organized. Use and name folders and files clearly

## You shouldn’t have

- [ ] ANYTHING AI-GENERATED\! Especially graphics. It should feel high-effort and designed.
- [ ] Designs copied from other people. It’s okay to reference or use parts of other’s work. Make sure to credit it. Never present others’ work as your own.
- [ ] Missing files\! Check the above

## DON'T FRAUD

Any project that includes stolen content, fully AI-generated design files, or other fraudulent/dishonest material may be permanently rejected and could result in a ban from Hackxpansion and other Hack Club programs.

## Examples

Here are some examples of well-shipped projects made by Hack Clubbers\! Notice how their READMEs are clear, organized repositories, and clear journals.

<html>
  <div class="grid! grid-cols-3 gap-4 text-center">
    <div>
      <img src="https://github.com/notaroomba/cyberboard/raw/main/assets/banner.png" alt="NotARoomba’s Cyberboard" />
      <a href="https://github.com/notaroomba/cyberboard">NotARoomba’s Cyberboard</a>
    </div>
    <div>
      <img src="https://github.com/cheyao/icepi-zero/raw/main/gallery/icepi-in-hand.png" alt="KOEGlike’s Split Keyboard" />
      <a href="https://github.com/KOEGlike/mito">Cyao's Icepi Zero</a>
    </div>
    <div>
      <img src="https://github.com/KaiPereira/Cheetah-MX4-Mini/raw/master/renders/real_photo.png" alt="Kai’s Cheetah MX4 Mini" />
      <a href="https://github.com/KaiPereira/Cheetah-MX4-Mini">Kai’s Cheetah MX4 Mini</a>
    </div>
  </div>
</html>

<br/>
<br/>
<br/>

> _This page is heavily inspired from [Fallout](https://github.com/hackclub/fallout/tree/main/docs/requirements)_
