'use client';
import {Checkbox, packSx, ScrollArea, ScrollAreaProps, Sx, Table, TextInput} from "@mantine/core";
import React, {useEffect, useMemo, useState} from "react";
import {PathText} from "./PathText";
import {useTableStyles} from "../utils";

export interface SelectionTableProps extends Omit<ScrollAreaProps, "onChange"> {
  data: Array<string>,
  limit: number,
  selection: Array<string>,
  onTruncated: (c: number | null) => void,
  sx?: Sx | Sx[],
  onChange: React.Dispatch<React.SetStateAction<Array<string>>>,
}

export const SelectionTable = React.memo(({
                                            data,
                                            selection,
                                            limit,
                                            onTruncated,
                                            onChange,
                                            sx,
                                            ...props
                                          }: SelectionTableProps) => {
  const {classes, cx} = useTableStyles();

  const allSelected = selection.length === data.length;
  const toggleAll = () => {
    onChange(allSelected ? [] : data)
  }
  const toggle = useMemo(() => (item: string) => {
    onChange((sel) => sel.includes(item) ? sel.filter(i => i !== item) : [...sel, item])
  }, []);

  const [filter, setFilter] = useState("");
  const filtered = data.filter(i => i.toLowerCase().includes(filter.toLowerCase()));

  useEffect(() => {
    onTruncated(filtered.length > limit ? filtered.length : null);
  }, [filtered, limit]);

  return (
    <ScrollArea sx={[{height: "100%"}, ...packSx(sx)]} styles={{scrollbar: {zIndex: 20}}} {...props}>
      <Table sx={{tableLayout: "fixed"}}>
        <thead className={cx(classes.stickyHeader)} style={{zIndex: 10}}>
        <tr>
          <th style={{width: 40}}>
            <Checkbox
              onChange={toggleAll}
              checked={allSelected}
              indeterminate={selection.length > 0 && !allSelected}
              transitionDuration={0}
            />
          </th>
          <th>
            <TextInput
              size={"xs"}
              value={filter}
              onChange={ev => setFilter(ev.currentTarget.value)}
              placeholder={"Filter..."}
            />
          </th>
        </tr>
        </thead>
        <tbody>
        {filtered.slice(0, limit).map(item => {
          const selected = selection.includes(item);
          return (<SelectionRow key={item} selected={selected} item={item} onToggle={toggle}/>)
        })}
        </tbody>
      </Table>
    </ScrollArea>
  )
});

type SelectionRowProps = {
  selected: boolean,
  item: string,
  onToggle: (item: string) => void
}

const SelectionRow = React.memo(({selected, item, onToggle}: SelectionRowProps) => {
  console.log("SelectionRow", item);
  const {classes, cx} = useTableStyles();

  return (
    <tr key={item} className={cx({[classes.rowSelected]: selected})}>
      <td>
        <Checkbox styles={{body: {marginTop: "auto"}}}
                  checked={selected} onChange={() => onToggle(item)} transitionDuration={0}/>
      </td>
      <td>
        <PathText keepFirst={4} keepLast={2} path={item} lineClamp={1} withinPortal/>
      </td>
    </tr>
  );
});

