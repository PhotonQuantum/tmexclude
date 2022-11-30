'use client';
import {Text, TextProps, Tooltip} from "@mantine/core";
import {useIsOverflow} from "../../../utils";
import {useMergedRef} from "@mantine/hooks";

export interface TipTextProps extends TextProps {
  ref?: any;
  withinPortal?: boolean
}

export const TipText = ({
                          ref,
                          children,
                          withinPortal,
                          ...props
                        }: TipTextProps) => {
  const {
    ref: overflowRef,
    isOverflow
  } = useIsOverflow();
  const mergedRef = useMergedRef(ref, overflowRef);
  return (<Tooltip label={children} multiline disabled={!isOverflow} withinPortal={withinPortal}>
    <Text ref={mergedRef} {...props}>{children}</Text>
  </Tooltip>)
}
