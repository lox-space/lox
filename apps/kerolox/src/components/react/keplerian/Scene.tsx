// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { useLox } from "@lox-space/react";
import { useEffect } from "react";

const Scene = () => {
  const lox = useLox();

  useEffect(() => {
    const foo = lox.deg_to_rad(180);
    console.log(foo);
  }, []);

  return <></>;
};

export default Scene;
