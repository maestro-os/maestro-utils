#include <crypt.h>
#include <string.h>

/*
 * Checks the given password `pass` against the given hashed password `hashed`.
 */
int check_pass(const char *pass, const char *hashed)
{
	struct crypt_data data;
	data.initialized = 0;

	char *result = crypt_r(pass, hashed, &data);
	return (strcmp(result, hashed) == 0);
}
