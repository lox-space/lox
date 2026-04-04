# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox


def main():
    mu = lox.Origin("Earth").gravitational_parameter()

    print(
        "Hello, Earthling!\n",
        f"The gravitational parameter of your planet is {mu} km^3/s^2.",
    )


if __name__ == "__main__":
    main()
