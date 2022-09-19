import {Sx, Title as MantineTitle, TitleProps as MantineTitleProps} from "@mantine/core";

type PartialSx = Omit<Sx, "userSelect" | "cursor">;

export type TitleProps = Omit<MantineTitleProps, "sx"> & {
    sx?: PartialSx;
};

export const Title = (props: TitleProps) => {
    const {sx, ...rest} = props;
    let newSx: Sx = {
        userSelect: "none",
        cursor: "default",
        ...props.sx
    };
    return <MantineTitle {...rest} sx={newSx}/>
}