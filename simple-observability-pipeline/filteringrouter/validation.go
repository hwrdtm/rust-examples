package filteringrouter

import "fmt"

// / Returns an error if the string does not begin with the prefix.
func validateStringBeginsWith(str string, prefix string) error {
	validates := str[:len(prefix)] == prefix

	if !validates {
		return fmt.Errorf("String %s... does not begin with prefix %s", str[:30], prefix)
	}

	return nil
}
