# S07: current32 기본 embedded font 실제 바이트 감사

- 상태: completed-scoped
- 분류: validation
- 우선순위: P0
- 요구사항: R11, S07
- 담당/수정 범위: search_reveal; 이 작업 문서와 `.local-tools/session-recovery-20261013/current32-embedded-font-audit-20261016/`만
- 선행 작업: current32 whole/graph/package/selected notice·SBOM

## 사용자 결과

기본 폰트 원자료와 실제 출하 CPU/installer PE 사이에 완전한 원 font byte가 포함되는지 확인하고 해당 고지 연결을 검토 가능하게 만든다.

## 계약과 범위

원 `.crate` SHA/lock, include_bytes source, 선택 notice inventory와 원 fullPE를 bounded 읽기로 대조한다. 원문 존재·완전 byte match와 runtime 사용/모든 linked code/법적완결을 구분한다. 시스템 폰트 runtime 읽기는 배포 내장 폰트로 취급하지 않는다. native/Cargo/네트워크/다운로드/GUI/기존 자료 수정0. 후속 whole33에 현재32 결과를 자동 승격하지 않는다.

## 인수 조건

- 원 archive lock SHA와 archive 내 source/font/license가 cache와 byte 동일.
- 원 font 전부를 고정 current32 CPU/installer PE에서 bounded 완전 검색하고 offset/section/원 pin 기록. 이름 문자열만으로 포함을 판정하지 않음.
- 변조 font·PE를 성공으로 수용하지 않는 음수 검사.
- 원 package의 false flag를 수정하지 않고 새 좁은 evidence로 기록.

## 검증과 handoff

실제 `python3 -B .local-tools/session-recovery-20261013/current32-embedded-font-audit-20261016/audit_fonts.py --generate`0, `--check`0. whole32 CPU `.rdata`에4개 font 전체 원문/SHA 일치 및 각 font notice의 archive→cache→portable→installed frame 연결을 확인했다. 원 font와 포함 PE의 메모리1byte 변조2개를 거절했다. 원 파일 변경0.

installer/CLI에서 동일4원자료 full sequence는0회이나 모든 변형/다른 font 부재로 일반화하지 않는다. 시스템 font는 소스상 runtime 읽기/GDI 선택이며 실제 시스템 font 파일을 읽거나 배포자료로 복사하지 않았다.

산출물은 담당 namespace의 `font-audit-result.json`, `audit_fonts.py`, `REPORT.md`, `HANDOFF.md`다. 기존 package false flags를 변경하지 않은 좁은 긍정 evidence이며 runtimeglyph·모든embedded·법적완비·공급trust·서명·공개배포는 미인수다. source/PE가 다른 whole33으로 결과를 승격하지 않는다. Cargo/native/network/GUI/삭제0. 문서 검사는 Root 최종통합에서 수행한다.


Root는 같은 auditor --check를 별도로 실제 exit0으로 재실행하고4개/변조2거절 결과와 source/CPU·notice 범위를 대조했다. [Root 고정 검토](../validation/evidence/current32-embedded-font-root-20261016/root-review.json). 설치 payload에 전달되는 framed 고지를 대조했으며 이 감사가 새 OS 설치를 실행한 것은 아니다. 현재false package flag/원 ZIP은 역사적 원본으로 보존한다.
