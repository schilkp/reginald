from typing import List

from reginald.datamodel import Docs


def doxy_comment(docs: Docs, prefix: str) -> List[str]:
    brief = docs.brief
    doc = docs.doc

    if brief is not None:
        brief = brief.strip()
        if len(brief) == 0:
            brief = None
    if doc is not None:
        doc = doc.strip()
        if len(doc) == 0:
            doc = None

    if brief is not None and doc is None:
        return [f"{prefix}/** @brief {brief} */"]
    elif doc is not None:
        l = []
        l.append(f"{prefix}/**")
        if brief is not None:
            l.append(f"{prefix} * @brief {brief}")
        for line in doc.splitlines():
            l.append(f"{prefix} * {line}")
        l.append(f"{prefix} */")
        return l
    else:
        return []
