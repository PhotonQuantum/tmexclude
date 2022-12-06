'use client';
import {Accordion, Box, Button, Container, Group, Popover, ScrollArea, Stack, Text, TextInput} from "@mantine/core";
import {IconPlus, IconTemplate} from "@tabler/icons";
import {useRecoilValue, useSetRecoilState} from "recoil";
import {allPathsState, ruleNamesState, rulesState} from "../../states";
import {useState} from "react";
import {useElementSize} from "@mantine/hooks";
import {RuleItem} from "../../components/RuleItem";

const AddButton = () => {
  const [addPop, setAddPop] = useState(false);
  const [name, setName] = useState("");
  const ruleNames = useRecoilValue(ruleNamesState);
  const setRules = useSetRecoilState(rulesState);
  const {
    ref,
    width
  } = useElementSize();
  const validateName = (name: string) => (name !== "" && !ruleNames.includes(name));
  const confirmName = () => {
    if (validateName(name)) {
      setRules((rules) => {
        return {
          ...rules,
          [name]: []
        }
      });
      return true;
    }
    return false;
  };
  return (<Popover withArrow trapFocus shadow={"sm"} opened={addPop} onChange={setAddPop} position={"bottom-end"}
                   arrowOffset={width / 2}>
    <Popover.Target>
      <Button ref={ref} variant={"default"} size={"xs"} leftIcon={<IconPlus size={16}/>}
              onClick={() => setAddPop(true)}>
        Add Rule
      </Button>
    </Popover.Target>
    <Popover.Dropdown mr={"md"}>
      <TextInput size={"xs"} placeholder={"Rule name"} value={name} error={!validateName(name) && name !== ""}
                 onChange={(ev) => {
                   setName(ev.currentTarget.value);
                 }}
                 onKeyDown={(ev) => {
                   if (ev.key === "Enter") {
                     if (confirmName()) {
                       setAddPop(false);
                       setName("");
                       ev.preventDefault();
                     }
                   } else if (ev.key === "Escape") {
                     setAddPop(false);
                     ev.preventDefault();
                   }
                 }}
      />
    </Popover.Dropdown>
  </Popover>)
}

export const Rules = () => {
  const ruleNames = useRecoilValue(ruleNamesState);
  const rules = useRecoilValue(rulesState);
  const allPaths = useRecoilValue(allPathsState);
  return (<Container sx={{height: "100%"}}>
    <Stack py={"xl"} sx={{height: "100%"}}>
      <Group sx={{flexWrap: "nowrap"}}>
        <Box><IconTemplate size={32} strokeWidth={1}/></Box>
        <Box sx={{flexGrow: 1}}>
          <Text>Exclude paths that match these patterns</Text>
          <Text size={"sm"} color={"dimmed"}>
            Patterns must be applied by at least one directory to take effect.
          </Text>
        </Box>
        <AddButton/>
      </Group>
      <Box/>
      <ScrollArea sx={{height: "100%"}}>
        <Accordion
          variant={"filled"}
          radius={"xs"}
          chevronPosition={"left"}
          styles={(theme) => ({
            item: {
              borderBottomStyle: "solid",
              borderBottomWidth: "1px",
              borderBottomColor: theme.colorScheme === 'dark' ? theme.colors.dark[4] : theme.colors.gray[2],
            },
            control: {
              padding: theme.spacing.sm
            }
          })}
        >{Object.entries(rules)
          .sort(([n1, _1], [n2, _2]) => n1.localeCompare(n2))
          .map(([name, value]) => (
            <RuleItem key={name} name={name} value={value} allPaths={allPaths} ruleNames={ruleNames}/>))}
        </Accordion>
      </ScrollArea>
    </Stack>
  </Container>)
}
