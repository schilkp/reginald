import { SiGit } from "@icons-pack/react-simple-icons";
import { BookOpenText } from "lucide-react";

export function Header() {
  return (
    <div className="border-b p-2 flex items-center justify-between bg-background">
      <h1 className="text-xl font-bold">Reginald</h1>

      <div className="flex items-center space-x-2">
        {/* Github Repo */}
        <a
          href={__REGINALD_REPO__}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center justify-center h-10 w-10 m-0 rounded-md text-foreground hover:bg-muted hover:text-primary transition-colors"
          aria-label="Repository"
        >
          <SiGit className="h-5 w-5" />
        </a>
        {/* Docs */}
        <a
          href={__REGINALD_BASE__ + "docs/index.html"}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center justify-center h-10 w-10 rounded-md text-foreground hover:bg-muted hover:text-primary transition-colors"
          aria-label="Repository"
        >
          <BookOpenText className="h-5 w-5" />
        </a>
      </div>
    </div>
  );
}
