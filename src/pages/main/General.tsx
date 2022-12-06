'use client';
import {Box, Checkbox, Container, Stack, Text} from "@mantine/core";
import {useRecoilState} from "recoil";
import {autoStartState, noIncludeState} from "../../states";

export const General = () => {
  const [noInclude, setNoInclude] = useRecoilState(noIncludeState);
  const [autoStart, setAutoStart] = useRecoilState(autoStartState);
  return (<Container>
    <Stack py={"xl"}>
      <Checkbox size={"sm"} label={<Text size={"md"}>Start at Login</Text>}
                checked={autoStart} onChange={(ev) => {
        setAutoStart(ev.currentTarget.checked);
      }}/>
      <Box/>
      <Checkbox
        checked={noInclude}
        size={"sm"}
        onChange={() => {
          setNoInclude(!noInclude);
        }}
        label={<>
          <Text size={"md"}>Ignore included files</Text>
          <Text size={"sm"} color={"dimmed"}>
            Don't include files into backups even if they don't match the rules.
          </Text>
        </>}/>
    </Stack>
  </Container>)
};