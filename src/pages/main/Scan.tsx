import {useRecoilValue} from "recoil";
import {scanPageState, scanStepState} from "../../states";
import {Overview} from "./scan/Overview";
import {Container} from "@mantine/core";
import {Done} from "./scan/Done";
import {Welcome} from "./scan/Welcome";
import {Applying} from "./scan/Applying";
import {Detail} from "./scan/Detail";
import {AnimatePresence} from "framer-motion";
import {InProgress} from "./scan/InProgress";
import {Log} from "./scan/Log";

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
              scanPage === "done" ?
                <Done/> :
                <Log/>
        }
      </AnimatePresence>
    </Container>
  )
}