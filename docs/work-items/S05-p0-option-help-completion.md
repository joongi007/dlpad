# S05: P0 주요 옵션 도움말 누락 보완

- 상태: implemented / integration GUI verification pending
- 분류: feature
- 우선순위: P0
- 요구사항: R07
- 담당 파일: soak_recovery_audit — crates/app/src/catalog.rs, accessible_options.rs, accessible.rs, 이 문서
- 의존성: S05-p0-option-catalog-current-audit, 기존 typed plan와 native collapsing/focus scroll

## 범위와 인수 조건

품질·구간·자르기·FFmpeg·session cookie의 설명·예·기본·충돌·미확인 버전을 중앙 catalog에서 표시한다. 상세는 기존 native collapsing과 focus-scroll을 사용한다. Core/schema/dependency/main.rs는 수정하지 않으며 root가 의미 있는 옵션 회귀·CPU compile·격리 GUI를 검증한다. 문구 복제 test는 추가하지 않는다.

## 구현과 sourcefreeze

기존summary는유지하고quality/range/cut/FFmpeg/cookie추가상세를5개native collapsing header로출력한다. 모든예/default/conflict/version미확인문구는appcatalog중앙상수다. CPUFFMPEG_HELP누락을연결했다. 구간label은양쪽blank전체정책에맞췄고sharedmain.rs1160도같은중앙label을사용하므로문구변경만전파된다. main.rs는수정하지않았다.

`scripts/cpu_options_probe.py:72`는oldexactlabel을하드코딩하므로root또는별도harness담당이새label로업데이트해야한다. 이task는script를수정하지않았다. R07audit의coreOptionHelp확장제안대신root지시대로appcatalog만사용하여core/sharedschema를유지했다. URL/tool/output기본입력의추가help는이번명시5옵션scope밖후속이다.

소유3파일 `rustfmt --edition 2024` actualexit0; aliasEROFS로승인된single-taskformatter만require_escalated실행했다. SHA256: catalog.rs `2760ddf24784282d4d7db84cfe4528f4ae911dd7813e9b73d8ebcd6f4d681f36`, accessible_options.rs `73cc0c5469f6c1e56a2b313623605d5b975e0f4cb9d8696a43c21e7d22333d4a`, accessible.rs `481a00f93ed680be266cd3e66705cc5368508c71604ff5c08b4f518bd7c89984`.

Cargo/nativeGUI/meaningfuloptiontests는root실행대기다. Source형식성공을compile/실제도움말가시성PASS로바꾸지않는다. 960x640스크롤/키보드focus/header펼침및한정OS인수는잔여다. 새dependency/thread/schema/문구복제test는없다.

`python3 scripts/check_docs.py`: exit0,455 documents PASS. 소스는 동결했으며 root 통합 검증 결과를 후속 기록한다.
