'use client';
import {Accordion, ActionIcon, Group, Menu, MultiSelect, SegmentedControl, Stack, Text, TextInput} from "@mantine/core";
import {IconDots, IconPencil, IconTrash} from "@tabler/icons";
import {useSetRecoilState} from "recoil";
import {perRuleState, rulesState} from "../../states";
import React, {useState} from "react";
import {PreRule} from "../../../bindings/PreRule";
import _ from "lodash";

type RuleItemProps = {
  name: string, value: PreRule, allPaths: string[], ruleNames: string[],
}

export const RuleItem = React.memo(({
                                      name,
                                      value,
                                      allPaths,
                                      ruleNames
                                    }: RuleItemProps) => {
  console.log("rule item rerender", name);
  const setRules = useSetRecoilState(rulesState);
  const setValue = useSetRecoilState(perRuleState(name));
  const [renaming, setRenaming] = useState(false);
  const [newName, setNewName] = useState("");
  const [prev, setPrev] = useState<PreRule | null>(null);

  const startRename = (name: string) => {
    setNewName(name);
    setRenaming(true);
  };
  const validateName = (name: string) => {
    return name.length > 0 && !ruleNames.includes(name);
  }
  const finishRename = () => {
    if (newName !== name && validateName(newName)) {
      setRules((rules) => {
        let newRules = {
          ...rules,
          [newName.trim()]: rules[name]
        };
        delete newRules[name];
        return newRules
      });
      return true;
    }
    return false;
  };
  const deleteRule = () => {
    setRules((rules) => {
      let newRules = {...rules};
      delete newRules[name];
      return newRules
    });
  };
  const switchRuleType = (type: "merge" | "concrete") => {
    const getRuleType = (rule: PreRule) => Array.isArray(rule) ? "merge" : "concrete";
    if (getRuleType(value) !== type) {
      if (prev !== null && getRuleType(prev) === type) {
        let prevValue = prev;
        setPrev(value);
        setValue(prevValue);
      } else {
        setPrev(value);
        if (type === "merge") {
          setValue([]);
        } else {
          setValue({
            excludes: [],
            "if-exists": []
          });
        }
      }
    }
  };

  return (<Accordion.Item key={name} value={name}>
    <Group sx={{flexWrap: "nowrap"}} mr={"sm"} spacing={0}>
      <Accordion.Control>
        {renaming ? <TextInput
          autoFocus
          value={newName}
          size={"xs"}
          error={!validateName(newName) && newName !== name}
          onChange={(e) => setNewName(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              if (finishRename()) {
                setRenaming(false);
                e.preventDefault();
              }
            } else if (e.key == "Escape") {
              setRenaming(false);
            }
          }}
          onBlur={() => {
            finishRename();
            setRenaming(false);
          }}
        /> : <Text size={"sm"} sx={{cursor: "pointer"}}>{name}</Text>}
      </Accordion.Control>
      <Menu withinPortal>
        <Menu.Target>
          <ActionIcon size={"lg"}><IconDots size={16}/></ActionIcon>
        </Menu.Target>
        <Menu.Dropdown>
          <Menu.Item icon={<IconPencil size={14}/>} onClick={() => startRename(name)}>Rename</Menu.Item>
          <Menu.Item color={"red"}
                     icon={<IconTrash size={14}/>}
                     onClick={deleteRule}>Delete</Menu.Item>
        </Menu.Dropdown>
      </Menu>
    </Group>
    <Accordion.Panel>
      <Stack spacing={"xs"}>
        <SegmentedControl
          size={"xs"}
          data={[{
            label: "Merge Rule",
            value: "merge"
          }, {
            label: "Concrete Rule",
            value: "concrete"
          }]}
          value={Array.isArray(value) ? "merge" : "concrete"}
          onChange={(e) => switchRuleType(e as "merge" | "concrete")}
        />
        {Array.isArray(value) ? (<MultiSelect
          searchable
          data={ruleNames.filter((k) => k !== name)}
          value={value}
          onChange={(newMergeRule) => {
            setValue(newMergeRule);
          }}
          placeholder={"Pick all sub-rules to merge"}
        />) : (<>
          <Text size="sm">Paths to exclude</Text>
          <MultiSelect searchable creatable
                       getCreateLabel={(value) => `+ New ${value}`}
                       data={allPaths.map((v) => ({
                         value: v,
                         label: v
                       }))}
                       value={value.excludes}
                       onChange={(newExcludes) => {
                         setValue({
                           excludes: newExcludes,
                           "if-exists": value["if-exists"]
                         });
                       }}
          />
          <Text size="sm">
            ... only if any of these paths exists in the same directory
          </Text>
          <MultiSelect searchable creatable
                       getCreateLabel={(value) => `+ New ${value}`}
                       data={allPaths.map((v) => ({
                         value: v,
                         label: v
                       }))}
                       value={value["if-exists"]}
                       onChange={(newIfExists) => {
                         setValue({
                           excludes: value.excludes,
                           "if-exists": newIfExists
                         });
                       }}
          />
        </>)}
      </Stack>
    </Accordion.Panel>
  </Accordion.Item>)
}, _.isEqual);