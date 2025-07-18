import { styled } from "styled-components";

export const Column = styled.div<{ gap?: number; reverse?: boolean }>`
    display: flex;
    flex-direction: ${props => props.reverse ? 'column-reverse' : 'column'};
    align-items: center;
    gap: ${props => props.gap || 0}px;
`; 