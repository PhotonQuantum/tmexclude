import {Sx, Text as MantineText, TextProps as MantineTextProps} from "@mantine/core";

type PartialSx = Omit<Sx, "userSelect" | "cursor">;

export type TextProps = Omit<MantineTextProps, "sx"> & {
    sx?: PartialSx;
};

export const Text = (props: TextProps) => {
    const {sx, ...rest} = props;
    let newSx: Sx = {
        userSelect: "none",
        cursor: "default",
        ...props.sx
    };
    return <MantineText {...rest} sx={newSx}/>
}