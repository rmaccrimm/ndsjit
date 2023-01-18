FROM stronglytyped/arm-none-eabi-gcc:latest

CMD ["arm-none-eabi-as", "-o", "/dev/nul", "-al", "--no-warn", "-mbig-endian"]

