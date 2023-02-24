#include <crypt.h>
#include <stdlib.h>
#include <string.h>

// TODO
/*
 * Hashes the given password.
 *
 * The function generates a random salt for each password.
 */
/*char *hash_pass(const char *pass) {
	struct crypt_data data;
	data.initialized = 0;

	char *setting = crypt_gensalt("$y$", 5, NULL, 0);
	char *output = crypt_r(pass, setting, &data);
	size_t len = strlen(output);

	char *allocated = malloc(len);
	memcpy(allocated, output, len);

	return allocated;
}*/

/*
 * Checks the given password `pass` against the given hashed password `hashed`.
 */
int check_pass(const char *pass, const char *hashed)
{
	struct crypt_data data;
	data.initialized = 0;

	char *output = crypt_r(pass, hashed, &data);
	return (strcmp(output, hashed) == 0);
}
