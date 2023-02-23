#include <crypt.h>
#include <string.h>

/*
 * Hashes the given password.
 *
 * The function generates a random salt for each password.
 */
char *hash_pass(const char *pass) {
	struct crypt_data data;
	data.initialized = 0;

	char *setting = crypt_gensalt("$y$", 5, NULL, 0);
	char *output = crypt_r(pass, setting, &data);

	char *allocated = malloc(sizeof(data.output));
	memcpy(allocated, output, sizeof(data.output));

	return allocated;
}

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
