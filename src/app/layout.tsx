import {ReactNode} from "react";
import {Providers} from "./providers";
import "./global.css";

const RootLayout = ({children}: { children: ReactNode }) => (
  <html lang="en">
  <head>
    <title>tmexclude</title>
    <meta name="viewport" content="minimum-scale=1, initial-scale=1, width=device-width"/>
  </head>

  <body style={{
    backgroundColor: "transparent",
    overflow: "hidden"
  }}>
  <Providers>{children}</Providers>
  </body>
  </html>
);

export default RootLayout;