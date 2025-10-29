CFLAGS = -Wall -Wextra -pedantic -O3 -g0 -llto

all: mocklinux

mocklinux: main.c
	$(CC) -o $@ $<
	strip $@

clean:
	rm -f mocklinux

.PHONY: all clean