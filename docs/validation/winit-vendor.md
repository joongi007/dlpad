# winit0.30.13 프로젝트 수정본

2026-10-02. [작업 계약](../work-items/S00-winit-startless-preedit.md). 최종스택채택아닌실증중패치다.

원본은Cargo.lock에고정된공식crates.io winit0.30.13이고프로젝트전용Cargo registry/cache에서복사했다. 원crate SHA256: a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d. 원LICENSE를그대로보존했다. 출처https://crates.io/crates/winit/0.30.13 이며새네트워크전송/새패키지는없다. vendor는사용자홈/toolcache에의존하지않고프로젝트안에있다.

변경은X11 event_processor의Start없이온nonemptyUpdate를composition으로인식하는것뿐이다. Xlib reference는서버draw의encoding_is_wchar=0, Start미관측, Draw/Done/commit을독립검증했다. 따라서widechar무시branch나XFilterEvent의None/eventwindow동등성을문제원인으로가정해패치하지않았다. XFilterEvent의None는event구조window를적용한다는설명은 [X.Org Xlib표준](https://www.x.org/releases/X11R7.6/doc/libX11/specs/libX11/libX11.pdf)에따른다.

Cargo patch경로와workspaceexclude를추가했고lock을기존version으로갱신한다. source약3MiB/추가crate없음/기존bool assignment이며native실측뒤비용과실패결과를기록한다. 기존GNUWindows및GUI증거는이전snapshot으로유지하고새동작을증명한것으로확대하지않는다.
