import {Box, Container} from "@mantine/core";
import {motion} from "framer-motion";
import {useElementSize, useViewportSize} from "@mantine/hooks";
import {NavBar} from "./NavBar";
import {MainHeader} from "./Header";
import {Outlet, useLocation} from "react-router-dom";

const variants = {
  hidden: {opacity: 0},
  enter: {opacity: 1},
  exit: {opacity: 0},
}

export const MainLayout = () => {
  const location = useLocation();
  const {ref, height: headerHeight} = useElementSize();
  const {height: vh} = useViewportSize();
  return (<Box sx={{
    display: "flex",
    flexDirection: "row"
  }} m={"auto"}>
    <NavBar/>
    <Box sx={{
      width: "100%",
      display: "flex",
      flexDirection: "column",
      overflowX: "hidden"
    }}>
      <MainHeader ref={ref}/>
      <Container
        sx={(theme) => ({
          backgroundColor: theme.colorScheme === 'dark' ? theme.colors.dark[8] : theme.colors.gray[0],
          height: vh - headerHeight - theme.spacing.xs * 2 - 1,
          width: "100%"
        })}>
        <motion.div
          key={location.pathname}
          variants={variants}
          initial={"hidden"}
          animate={"enter"}
          exit={"exit"}
          style={{height: "100%"}}
        >
          <Outlet/>
        </motion.div>
      </Container>
    </Box>
  </Box>)
};