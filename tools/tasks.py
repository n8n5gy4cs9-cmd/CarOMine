#!/usr/bin/env python3
"""PRD.json is the source of truth. Usage:
  tools/tasks.py render                 -> regenerate task.md
  tools/tasks.py done T-010 ["note"]    -> mark done + render
  tools/tasks.py wip T-010 ["note"]     -> mark in-progress + render
  tools/tasks.py add P3 T-049 "title"   -> add task + render
  tools/tasks.py next                   -> print next todo/wip task
"""
import json,sys,pathlib
R=pathlib.Path(__file__).resolve().parent.parent
J=R/"PRD.json"
M={"done":"[x]","wip":"[~]","todo":"[ ]"}
def load(): return json.load(open(J))
def save(d): json.dump(d,open(J,"w"),indent=2,ensure_ascii=False); render(d)
def tasks(d): return [(p,t) for p in d["phases"] for t in p["tasks"]]
def render(d=None):
    d=d or load(); a=tasks(d); n=len(a); k=sum(t["status"]=="done" for _,t in a)
    L=["# CarOMine - TASKS","",f"> Generated from PRD.json by tools/tasks.py - do not hand-edit. **{k}/{n} done.**",
       "> Legend: `[x]` done, `[~]` in progress, `[ ]` not started.","",
       "## NEXT UP",""]
    nx=[(p,t) for p,t in a if t["status"]!="done"][:5]
    L+= [f"- {M[t['status']]} **{t['id']}** {t['title']}" for p,t in nx] or ["- (all done)"]
    for p in d["phases"]:
        dn=sum(t["status"]=="done" for t in p["tasks"])
        L+=["",f"## {p['id']} {p['name']} ({dn}/{len(p['tasks'])})",""]
        for t in p["tasks"]:
            L.append(f"- {M[t['status']]} **{t['id']}** {t['title']}"+(f"  \n  _note: {t['notes']}_" if t["notes"] else ""))
    (R/"task.md").write_text("\n".join(L)+"\n")
def main():
    c=sys.argv[1] if len(sys.argv)>1 else "render"; d=load()
    if c=="render": render(d)
    elif c in("done","wip"):
        for _,t in tasks(d):
            if t["id"]==sys.argv[2]:
                t["status"]=c
                if len(sys.argv)>3: t["notes"]=sys.argv[3]
        save(d)
    elif c=="add":
        for p in d["phases"]:
            if p["id"]==sys.argv[2]: p["tasks"].append({"id":sys.argv[3],"title":sys.argv[4],"status":"todo","notes":""})
        save(d)
    elif c=="next":
        for p,t in tasks(d):
            if t["status"]!="done": print(t["id"],t["title"]); break
main()
