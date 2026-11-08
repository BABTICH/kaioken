import { Layers } from "lucide-react";

interface SurfacePlaceholderProps {
  index?: string;
  title: string;
  phase: string;
  description: string;
}

export function SurfacePlaceholder({
  title,
  phase,
  description
}: SurfacePlaceholderProps) {
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="surface-card">
        <div className="surface-head flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="surface-num flex items-center justify-center">
              <Layers size={16} />
            </span>
            <h2 className="surface-title">{title}</h2>
          </div>
          <span className="surface-note">{phase}</span>
        </div>

        <div className="surface-body">
          <p className="lead">{description}</p>
        </div>
      </div>
    </div>
  );
}
