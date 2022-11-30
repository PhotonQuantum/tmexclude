'use client';
import {Text, TextProps, Tooltip} from "@mantine/core";
import {useIsOverflow, useTruncatedPath} from "../utils";
import {useMergedRef} from "@mantine/hooks";

export interface PathTextProps extends TextProps {
  keepFirst: number;
  keepLast: number;
  ref?: any;
  withinPortal?: boolean;
  path: string;
}

export const PathText = ({
                           keepFirst,
                           keepLast,
                           path,
                           ref,
                           withinPortal,
                           ...props
                         }: PathTextProps) => {
  const {
    ref: overflowRef,
    isOverflow
  } = useIsOverflow();
  const mergedRef = useMergedRef(ref, overflowRef);
  const [truncated, truncatedPath] = useTruncatedPath(path, keepFirst, keepLast);
  return (<Tooltip label={path} multiline disabled={!truncated && !isOverflow} withinPortal={withinPortal}>
    <Text ref={mergedRef} {...props}>{truncatedPath}</Text>
  </Tooltip>)
}