import * as React from "react"

export function useReducedMotion(): boolean {
  const [reduced, setReduced] = React.useState(false)
  React.useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)")
    const update = () => setReduced(mq.matches)
    update()
    mq.addEventListener("change", update)
    return () => mq.removeEventListener("change", update)
  }, [])
  return reduced
}

export function useInView<T extends Element>(
  ref: React.RefObject<T | null>,
  rootMargin = "0px"
): boolean {
  const [inView, setInView] = React.useState(false)
  React.useEffect(() => {
    const el = ref.current
    if (!el) return
    const obs = new IntersectionObserver(
      ([entry]) => setInView(entry.isIntersecting),
      { rootMargin, threshold: 0.15 }
    )
    obs.observe(el)
    return () => obs.disconnect()
  }, [ref, rootMargin])
  return inView
}

export function useTypewriter(text: string, run: boolean, speed = 26): string {
  const reduced = useReducedMotion()
  const [shown, setShown] = React.useState(reduced ? text : "")

  React.useEffect(() => {
    if (reduced || !run) {
      setShown(reduced ? text : "")
      return
    }
    setShown("")
    let i = 0
    const id = window.setInterval(() => {
      i += 2
      setShown(text.slice(0, i))
      if (i >= text.length) window.clearInterval(id)
    }, speed)
    return () => window.clearInterval(id)
  }, [text, run, speed, reduced])

  return shown
}
