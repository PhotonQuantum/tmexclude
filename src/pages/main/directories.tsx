import {getMainLayout} from "../../components/mainLayout";
import {
  Badge, Box, Button, Container, createStyles, MultiSelect, Popover, ScrollArea, Stack, Table, Text, Tooltip
} from "@mantine/core";
import {IconMinus, IconPlus} from "@tabler/icons";
import React, {useEffect, useState} from "react";
import {useRecoilState, useRecoilValue, useSetRecoilState} from "recoil";
import {dirsState, perDirState, ruleNamesState, skipsState} from "../../states";
import {open} from "@tauri-apps/api/dialog";
import {truncatePath} from "../../utils";
import _ from "lodash";

const buttonStyles = {
  root: {paddingRight: 7},
  leftIcon: {marginRight: 0}
};

const useStyles = createStyles((theme) => ({
  rowSelected: {
    backgroundColor: theme.colorScheme === 'dark' ? theme.fn.rgba(theme.colors[theme.primaryColor][7],
      0.2) : theme.colors[theme.primaryColor][0],
  },
}));

type WatchedDirItemProps = {
  path: string, rules: string[], ruleNames: string[]
}

const WatchedDirItem = React.memo(({
                                     path,
                                     rules,
                                     ruleNames
                                   }: WatchedDirItemProps) => {
  const setValue = useSetRecoilState(perDirState(path));
  const [[truncated, truncatedPath], setTruncated] = useState<[boolean, string]>([false, path]);
  const setDirs = useSetRecoilState(dirsState);
  useEffect(() => {
    truncatePath(path).then((truncated: [boolean, string]) => {
      setTruncated(truncated);
    });
  }, [path]);

  const removeDir = () => {
    setDirs((dirs) => {
      return dirs.filter((dir) => dir.path !== path);
    });
  }
  return (<Popover withArrow trapFocus withinPortal shadow={"sm"} width={500}>
    <Popover.Target>
      <Box component="tr" key={path} sx={{cursor: "pointer"}}>
        <td>
          <Tooltip label={path} disabled={!truncated}>
            <Text sx={{
              cursor: "pointer",
              minWidth: 70
            }}>{truncatedPath}</Text>
          </Tooltip>
        </td>
        <td>{rules.map((rule) => (<Badge key={rule} variant={"light"}>
          <Text size={9} sx={{cursor: "pointer"}}>{rule}</Text>
        </Badge>))}</td>
      </Box>
    </Popover.Target>
    <Popover.Dropdown>
      <>
        <Box data-autofocus/>
        <Stack spacing={"xs"}>
          <MultiSelect
            searchable
            data={ruleNames}
            value={rules}
            maxDropdownHeight={150}
            placeholder={"Pick rules to apply"}
            onChange={(rules) => {
              setValue((value) => {
                return {
                  ...value,
                  rules: rules
                };
              })
            }}
          />
          <Button color={"red"} onClick={removeDir}>Remove Directory</Button>
        </Stack>
      </>
    </Popover.Dropdown>
  </Popover>)
}, _.isEqual);

const WatchedDir = () => {
  const ruleNames = useRecoilValue(ruleNamesState);
  const [dirs, setDirs] = useRecoilState(dirsState);
  const addDir = async () => {
    const selected = await open({
      directory: true
    });
    if (typeof selected === "string") {
      setDirs((dirPaths) => {
        if (dirPaths.find((dir) => dir.path === selected)) {
          return dirPaths;
        }
        return [...dirPaths, {
          path: selected,
          rules: []
        }];
      });
    }
  }
  return (<>
    <Text>Directories to watch and scan</Text>
    <ScrollArea sx={{height: "40%"}}>
      <Table highlightOnHover>
        <tbody>
        {dirs.map(({
                     path,
                     rules
                   }) => (<WatchedDirItem path={path} rules={rules} ruleNames={ruleNames}/>))}
        </tbody>
      </Table>
    </ScrollArea>
    <Button ml={"auto"} size={"xs"} compact leftIcon={<IconPlus size={12}/>} styles={buttonStyles}
            variant={"default"}
            onClick={addDir}
    />
  </>)
}

const SkippedDirs = () => {
  const [skipped, setSkipped] = useRecoilState(skipsState);
  const [selected, setSelected] = useState(-1);
  const {
    classes,
    cx
  } = useStyles();
  const addDir = async () => {
    const selected = await open({
      directory: true
    });
    if (typeof selected === "string") {
      setSkipped((skipped) => {
        if (skipped.includes(selected)) {
          return skipped;
        }
        return [...skipped, selected];
      });
    }
  }
  const mayRemove = selected >= 0 && selected < skipped.length;
  const removeDir = (index: number) => {
    setSkipped((skipped) => {
      return skipped.filter((_, i) => i !== index);
    });
  };
  return (<>
    <Text>Skip the following paths</Text>
    <ScrollArea sx={{height: "40%"}}>
      <Table highlightOnHover m={"auto"}>
        <tbody>
        {skipped.map((path, idx) => (
          <tr key={path} onClick={() => setSelected(idx)} className={cx({[classes.rowSelected]: selected === idx})}>
            <td>
              <Text>{path}</Text>
            </td>
          </tr>))}
        </tbody>
      </Table>
    </ScrollArea>
    <Button.Group ml={"auto"}>
      <Button size={"xs"} compact leftIcon={<IconPlus size={12}/>} styles={buttonStyles} variant={"default"}
              onClick={addDir}/>
      <Button size={"xs"} compact leftIcon={<IconMinus size={12}/>} styles={buttonStyles} variant={"default"}
              disabled={!mayRemove}
              onClick={() => removeDir(selected)}/>
    </Button.Group>
  </>)
}

const Directories = () => {
  return (<Container sx={{height: "100%"}}>
    <Stack py={"xl"} sx={{height: "100%"}}>
      <WatchedDir/>
      <SkippedDirs/>
    </Stack>
  </Container>)
}

Directories.getLayout = getMainLayout;

export default Directories;
