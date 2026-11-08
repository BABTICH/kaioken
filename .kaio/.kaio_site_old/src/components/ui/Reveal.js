import { jsx as _jsx } from "react/jsx-runtime";
import * as React from "react";
import { useInView } from "@/lib/motion";
import { cn } from "@/lib/utils";
export default function Reveal({ children, delay = 0, className, as: Tag = "div", ...rest }) {
    const ref = React.useRef(null);
    const inView = useInView(ref, "-40px");
    const [shown, setShown] = React.useState(false);
    React.useEffect(() => {
        if (inView)
            setShown(true);
    }, [inView]);
    return (_jsx(Tag, { ref: ref, className: cn("reveal", shown && "reveal-in", className), style: delay ? { transitionDelay: `${delay}s` } : undefined, ...rest, children: children }));
}
