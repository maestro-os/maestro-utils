#include <termios.h>
#include <unistd.h>

struct termios get_termios()
{
	struct termios t;
	tcgetattr(STDIN_FILENO, &t);

	return t;
}

void set_termios(struct termios *t)
{
	tcsetattr(STDIN_FILENO, TCSANOW, t);
}
