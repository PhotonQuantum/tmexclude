'use client';
import {Box, Checkbox, Container, Group, Select, Stack, Text} from "@mantine/core";
import {useRecoilState} from "recoil";
import {noIncludeState} from "../../states";

export const General = () => {
  const [noInclude, setNoInclude] = useRecoilState(noIncludeState);
  return (<Container>
    <Stack py={"xl"}>
      <Checkbox size={"sm"} label={<Text size={"md"}>Start at Login</Text>}/>
      <Group>
        <Text>Language: </Text>
        <Select
          data={["Use system defaults", "English (US)", "Chinese Simplified"]}
          value={"Use system defaults"}
          size={"sm"}/>
      </Group>
      <Box/>
      <Checkbox
        checked={noInclude}
        size={"sm"}
        onChange={() => {
          console.log("change");
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