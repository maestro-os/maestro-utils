// Executes a command, making the Maestro kernel pass as Linux

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/utsname.h>
#include <unistd.h>

// `prctl` command: Maestro-specific subcommands
#define PR_MAESTRO			0x4d535452
// `PR_MAESTRO` subcommand: pretend to be Linux
#define PR_MAESTRO_LINUX	0

int main(int argc, char **argv) {
	struct utsname u;
	int res;

	if (argc <= 1) {
		dprintf(STDERR_FILENO, "usage: mocklinux <cmd> [args...]\n");
		return 1;
	}
	res = uname(&u);
	if (res < 0) {
		dprintf(STDERR_FILENO, "mocklinux: uname: error: %s\n", strerror(errno));
		return 1;
	}
    // If already Linux, do nothing as it may not be supported, or already being mocked
	if (strncmp(u.sysname, "Linux\0", 6) != 0) {
		res = prctl(PR_MAESTRO, PR_MAESTRO_LINUX, 1);
		if (res < 0) {
			dprintf(STDERR_FILENO, "mocklinux: prctl: error: %s\n", strerror(errno));
			return 1;
		}
	}
	execvp(argv[1], argv + 1);
	dprintf(STDERR_FILENO, "mocklinux: exec: error: %s\n", strerror(errno));
	return 1;
}