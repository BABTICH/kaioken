import * as Accordion from "@radix-ui/react-accordion"
import { ChevronDown } from "lucide-react"
import { FAQS } from "@/data/faq"
import Badge from "@/components/ui/Badge"
import Reveal from "@/components/ui/Reveal"

export default function FaqSection() {
  return (
    <section id="faq" className="py-24 border-b border-white/10 bg-[#050507]">
      <div className="mx-auto max-w-4xl px-4 sm:px-6">
        {/* Header */}
        <Reveal>
          <div className="mb-12 text-center">
            <Badge variant="dim" isComment className="mb-3">
              KNOWLEDGE BASE
            </Badge>
            <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
              Questions, answered.
            </h2>
            <p className="mx-auto mt-3 max-w-lg font-sans text-sm text-kai-muted">
              Everything you need to know about Kaioken's architecture, security model, and execution semantics.
            </p>
          </div>
        </Reveal>

        {/* Accordion */}
        <Reveal delay={0.1}>
          <Accordion.Root type="single" collapsible className="space-y-3">
            {FAQS.map((faq, idx) => (
              <Accordion.Item
                key={idx}
                value={`faq-${idx}`}
                className="overflow-hidden rounded-md border border-white/10 bg-[#0a0b10] transition-colors data-[state=open]:border-white/20 data-[state=open]:bg-[#0e0f16]"
              >
                <Accordion.Trigger className="flex w-full items-center justify-between p-5 text-left font-sans text-sm font-semibold text-white transition-all hover:text-kai-orange cursor-pointer group [&[data-state=open]>svg]:rotate-180">
                  <span>{faq.question}</span>
                  <ChevronDown className="size-4 shrink-0 text-kai-muted transition-transform duration-200 group-hover:text-white" />
                </Accordion.Trigger>

                <Accordion.Content className="overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
                  <div className="px-5 pb-5 font-sans text-xs text-kai-muted leading-relaxed border-t border-white/5 pt-3">
                    {faq.answer}
                  </div>
                </Accordion.Content>
              </Accordion.Item>
            ))}
          </Accordion.Root>
        </Reveal>
      </div>
    </section>
  )
}
