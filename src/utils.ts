import {createPolymorphicComponent, createStyles, Text as MantineText, TextProps} from "@mantine/core";
import styled from "@emotion/styled";

export const disableMenu = () => {
    if (typeof window === "undefined") {
        return;
    }

    // @ts-ignore
    if (window.__TAURI__.environment !== 'production') {
        return
    }

    document.addEventListener('contextmenu', e => {
        e.preventDefault();
        return false;
    }, {capture: true})

    document.addEventListener('selectstart', e => {
        e.preventDefault();
        return false;
    }, {capture: true})
}

export const useStyles = createStyles(() => ({
    prohibit: {
        userSelect: 'none',
        cursor: 'default',
    }
}));

const _Text = styled(MantineText)`
  // user-select: none;
  // cursor: default;
`;

export const Text = createPolymorphicComponent<'text', TextProps>(_Text);

export const ScrollBorder = createStyles((theme) => ({
    root:
        {
            maxHeight: "100%",
            borderStyle: "solid",
            borderWidth: "1px",
            borderRadius: theme.radius.xs,
            borderColor: theme.colorScheme === 'dark' ? theme.colors.dark[4] : theme.colors.gray[2]
        },
}));