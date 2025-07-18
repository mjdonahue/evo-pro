import { styled } from "styled-components";

export const Row = styled.div<{ gap?: number }>`
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: ${props => props.gap || 0}px;
`; 