# The Fi Project

### Project Phase Checklist

- [x] Pty forker
- [x] Ascii parser
- [ ] Link ascii parser to pty forker
- [ ] multiplexer?
- [ ] Implement Full Terminal Emulation
- [ ] Add Visual/Normal Modes and Commands
- [ ] TUI Library
- [ ] Plugin System
- [ ] Python configuration api?
- [ ] Incorperate JEF as plugin
- [ ] Text editor as plugin

### Problems to Solve

- [x] Fix blocking I/O on thread join issue
- [x] Figure out how to get child process pwd
- [x] Figure out where deadlock in forker is
    - [ ] Fix new thread join issue


### Dev Log
- **2024-06-07**: Official project start
- **2024-06-09**: Implemented simple ascii escape parser
- **2024-06-11**: Implemented basic pty forking functions
- **2024-06-15**: Minor fixes
