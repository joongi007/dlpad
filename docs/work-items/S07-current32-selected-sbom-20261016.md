# S07: current32 선택 Windows 구성요소 SBOM

- 상태: completed-scoped
- 분류: validation
- 우선순위: P0
- 요구사항: R11, S07
- 담당/수정 범위: search_reveal; 이 문서와 `.local-tools/session-recovery-20261013/current32-selected-sbom-audit-20261016/`만
- 선행 작업: whole32·선택 Windows graph·v32 portable/installer 원 package 검증

## 사용자 결과

현재 후보의 선택 의존성·원 lock checksum·선언 라이선스·포함 notice와 출하 PE를 하나의 검토 가능한 SBOM으로 연결한다.

## 계약과 범위

ADR 0001/0003/0005와 R11/S07을 따른다. Cargo/native/외부 네트워크/GUI/삭제/공유 schema 변경 없음. 원 ZIP/graph/lock/notice 보존. 선택 graph는 normal/build/proc-macro 상위 집합이며 정확한 linked-code 목록이 아니다. whole-lock43개 누락·공급 trust·법적 미인수를 보존하며 speculative 라이선스 선택을 하지 않는다.

## 인수 조건

- 실제 graph/metadata/lock과 selected notice inventory name/version 정확 일치.
- 원 ZIP 모든 notice SHA와 installer 묶음 framing/bytes 일치.
- 출하 binary/manifest/whole 연결 대조.
- 같은 입력에서 SBOM 재생성 후 validator byte 동일 확인.
- 전체 lock 누락을 선택 Windows payload 누락으로 오인하지 않고 embedded/tool 공급 의무 미완료 보존.

## 검증과 handoff

실제 `python3 -B .local-tools/session-recovery-20261013/current32-selected-sbom-audit-20261016/sbom.py --generate`가0으로 selected258/notice456/appendix43 SBOM을 생성했다. `--check` 재생성 대조도0이며 component 누락·binding 위조 두 음수 입력을 거절했다. 전체 lock 누락43과 선택 graph의 교집합0이다. 원 ZIP/소스/lock/notice 변경0, 새 의존성0, Cargo/native/network/GUI 실행0.

출력은 위 namespace의 `selected-current32.cdx.json`, `validation-result.json`, `REPORT.md`, 재현 validator `sbom.py`다. 기존 설치본/ZIP에 SBOM을 넣지는 않았다. 모든 자료는 sidecar 검토 범위이며 공식 CycloneDX schema 전체 적합성 검사는 미실행이다. 선택 graph는 정확한 linked-code SBOM이 아니다. 원문 보존·구성원 binding 인수와 법적완비/embedded payload/도구 공급 trust·서명·공개 배포 인수를 구분한다.

첫 재현 검사는675 source 전체 경로 읽기 지연으로 중단했고, Root 요청대로 최종 validator는 원 ZIP/graph/lock/whole binding의 bounded offline 검사로 제한했다. 전체 source before/after가 같은675 snapshot임을 원 proof끼리 대조하고 현재 Cargo.lock만 exact pin 대조한다. 새 whole workspace 검증이나 임의 feature 동등성 주장은 하지 않는다.
