'use client';
import {useRecoilValue} from "recoil";
import {scanPageState, scanStepState} from "../../states";
import {Overview} from "./pages/Overview";
import {Container} from "@mantine/core";
import {Done} from "./pages/Done";
import {Welcome} from "./pages/Welcome";
import {Applying} from "./pages/Applying";
import {Detail} from "./pages/Detail";
import {AnimatePresence} from "framer-motion";
import {InProgress} from "./pages/InProgress";

export const Scan = () => {
  const scanPage = useRecoilValue(scanPageState);
  const scanStep = useRecoilValue(scanStepState);

  return (
    <Container sx={{height: "100%"}}>
      <AnimatePresence mode={"popLayout"} initial={false}>
        {scanPage === "scan" ?
          (scanStep === "idle" ?
              <Welcome/> :
              scanStep === "scanning" ?
                <InProgress/> :
                <Overview/>
          ) :
          scanPage === "detail" ?
            <Detail/> :
            scanPage === "applying" ?
              <Applying/> :
              <Done/>}
      </AnimatePresence>
    </Container>
  )
}