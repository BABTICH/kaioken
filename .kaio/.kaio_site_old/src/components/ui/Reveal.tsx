import * as React from "react"
import { useInView } from "@/lib/motion"
import { cn } from "@/lib/utils"

export default function Reveal({
  children,
  delay = 0,
  className,
  as: Tag = "div",
  ...rest
}: {
  children: React.ReactNode
  delay?: number
  className?: string
  as?: "div" | "section"
} & React.HTMLAttributes<HTMLElement>) {
  const ref = React.useRef<HTMLDivElement>(null)
  const inView = useInView(ref, "-40px")
  const [shown, setShown] = React.useState(false)

  React.useEffect(() => {
    if (inView) setShown(true)
  }, [inView])

  return (
    <Tag
      ref={ref as React.Ref<HTMLDivElement>}
      className={cn("reveal", shown && "reveal-in", className)}
      style={delay ? { transitionDelay: `${delay}s` } : undefined}
      {...rest}
    >
      {children}
    </Tag>
  )
}
