# zerOS dev roadmap

## 17/11/2024

- [ ] finish first write of the memory management subsystem
- [ ] if possible, separate bootloader-specific requests / responses handling, and try to add support for more bootloaders. Also try to abstract bootloaders differences into a cleaner interface for retrieving responses, etc... (maybe load different modules depending on the detected bootloader ?) (maybe use custom binary format for kernel modules ?)
- [ ] setup per-cpu variables
- [ ] setup a stack for the kernel

## 01/12/2024

- [ ] get a working memory management subsystem
- [ ] get a working API for subsystems to register their interrupt handlers
- [ ] be able to write to the framebuffer (i.e. a proper `zerOS_printk` and some basic framebuffer related API)

## 15/12/2024

- [ ] make the kernel multicore-aware

## 29/12/2024

- [ ] start writing a task system, with a custom scheduler (?) (based on a multicore priority-based round robin scheduler)