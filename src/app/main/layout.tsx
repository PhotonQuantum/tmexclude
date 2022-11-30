import {ReactNode} from "react";
import {MainLayout} from "./MainLayout";

const Layout = ({children}: { children: ReactNode }) => (
  <MainLayout>{children}</MainLayout>
)

export default Layout;