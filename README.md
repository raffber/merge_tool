# Merge And Software Release Tool

This tool allows implementing a bare-metal firmware release process.

The tool takes 2 hex files as inputs: An application and a bootloader. Then it allows producing several artifacts out of them:

 * A merged hex file
 * A script file for updating the application
 * An JSON info file capturing release information about the image

As an extension of above features, it supports systems consisting of several "nodes" (i.e. individual MCUs). I.e. it allows processing several pairs of firmwares.

## Command Line Interface


## HEX File Layout

Some firmware meta-information is stored directly inline in the hex file. You may want to add it directly from your source code. Refer to an example here.


## Configuration File Format




## Things this tool does not do


 * It does not provide a cyrptographic signature with the image. Just a CRC32.
 * It does not encrypt the software.

 Pull requests to implement above features are welcome.

