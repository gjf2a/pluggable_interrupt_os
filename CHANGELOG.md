# **0.5.2 - 2025-3-7 **
  * Errors arose with soft-float. Changing the bootloader version to 0.9.30 
    and adding the line `"rustc-abi": "x86-softfloat"` to `x86_64-blog_os.json`
    seem to have fixed the errors.
# **0.5.1 - 2024-12-30 **
  * No change in functionality.
  * Updated documentation of using the template in README.md.
# **0.5 - 2024-12-20 **
  * Updated x86_64 to 0.15.2
  * Updated pc-keyboard to 0.8.0
  * Updated pic8259 to 0.11.0
  * Updated uart_16550 to 0.3.2
  * Fixed corresponding maintenance errors.
# **0.4**
  * Updated to pic8259 version 0.10
  * Fixed corresponding maintenance errors.
# **0.2**
  * Added `is_drawable()` function to determine whether a given `char` can be rendered in the VGA buffer.
  * Rewrote README.md to describe the [Pluggable Interrupt Template](https://github.com/gjf2a/pluggable_interrupt_template)   