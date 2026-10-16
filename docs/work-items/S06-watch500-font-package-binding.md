# S06: 현재 watch500 바이너리의 폰트와 개발 ZIP 연결

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R11, R13
- 담당/수정 범위: root — 기존 package/font verifier 실행·새 evidence·본 문서; metrics_recovery — /tmp 읽기 계획만
- 선행 작업: 현재 전체 Windows 검증·기본 CPU SHA 복원·CPU 측정 종료

## 사용자 결과

현재 제품 실행 파일과 함께 배포될 네 내장 폰트 및 공지문이 정확한 개발 ZIP에 연결되는지 검증한다.

## 계약과 범위

동일 CPU SHA59448518d46ae8f3da4bf18540ae8ed031497584f13799e231966dd28f754848와 source543 전체 proof로 fresh ZIP 두 개를 만들고 bounded 검토·full font byte/checksum/PE occurrence/notice·snapshot binding을 실행한다. 공개 배포·서명·설치·기존 사용자 데이터 변경은 없다. Cargo/schema 변경은 없다.

## 인수 조건

두 ZIP byte/SHA 동일, 원 실행 파일/공지 layout 및 상한, 네 폰트 원 bytes와 .rdata 정확 연결, 현재 proof/font result/verifier SHA 대응을 확인한다. 이전 snapshot 결과를 재사용하거나 누락 공지를 허용해 release_ready=true로 만들지 않는다.

## 검증과 handoff

하위가 기존 명령과 네 폰트 원자료를 확인했다. per-user installer·법적 완비·실제 tool trust·signing·clean machine/minimum OS는 이 검증으로 해결되지 않는다.

첫 package observer는 120초 한도로 중단돼 ZIP 미생성이며 원 로그를 보존했다. 소스 읽기에서 도구가 항상 explicit --allow-missing-licenses 개발 후보 옵션을 요구함을 확인했다. 이 옵션을 사용해도 release_ready=false와 license complete=false가 유지된다. 공개 배포 인수로 승격하지 않고 fresh 개발 artifact에 한해 600초 observer로 재실행한다. 실제 비용 원인은 아직 확정하지 않는다.

개발 ZIP 두 개가 byte/SHA 동일(28ae54b081608ed9f1babb33f4bed73c8923fd65070a21cbef0fd1eb361d840b, 12,711,814B, entries863)로 bounded 리뷰를 통과했다. 네 폰트 full byte/checksum/PE 정확 occurrence·원 notice와 현재 proof/verifier/font result/ZIP binding도 통과했다. [binding](../validation/evidence/windows-watch500-font-package-dev-20261006-continue/font-binding.json). license missing43은 전체 lock superset 값이며 Windows linked dependency 법적 완비를 의미하지 않는다. release_ready=false를 유지한다. 기존 binding 순수 실패 테스트 11개도 통과했다.

독립 검토를 원 개발 ZIP evidence에 복사했다. 두 ZIP의 전체SHA·863 entry digest/CRC·CPU/lock·네 전체폰트/공지문·font-binding 일치를 확인했다. 이 scoped 작업은 완료하며 release_ready=false와 wholelock notice 누락43개·실제 linked/legal 검토 미완료를 유지한다.
