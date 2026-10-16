# S07: Windows 개발 ZIP에 검증된 local-install CLI 포함

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — scripts/windows_current_workspace_probe.py artifact 계약·통합·검증·문서; diagnostic_cache — /tmp scripts/windows_candidate_package.py 초안; metrics_recovery — /tmp scripts/windows_candidate_package_review.py 초안; soak_recovery_audit — /tmp scripts/test_windows_companion_package.py 실패 회귀 초안
- 선행 작업: 일곱 명령 CLI 실제 인수 완료. 실행 중인 current notice tree의 source freeze가 끝나기 전 원 probe를 변경하지 않는다.

## 사용자 결과

선택한 개발 ZIP에 같은 Windows 전체검사에서 나온 GUI와 개발 local-install CLI를 함께 넣고 각 PE/bytes/SHA 및 정확 검사 증거를 대조한다. 두 ZIP을 같은 입력에서 동일하게 생성하며 서명 installer나 공개 release로 표시하지 않는다.

## 계약과 범위

probe는 전체4검사 후 `local_install_artifact={sha256,bytes}`를 CPU와 별도로 관측한다. packager의 선택적 `--local-install-exe ABS --local-install-sha256 SHA64`는 둘 다 제공하고 정확 proof artifact와 맞을 때만 `dlpad-local-install.exe`를 streaming64KiB로 포함한다. 이전 CPU-only/legacy proof 계약은 유지하며 CLI 없는 proof에서 companion을 추정하지 않는다. notice 문서 합16MiB·entries4096과 executables 합256MiB 상한을 유지한다. companion의 typed manifest는 development-only 명시 역할과 bytes/SHA/PE를 기록하고 source/입력/proof pre/post를 재검증한다. reviewer는 companion과 notice를 구별하고 정확 manifest/proof/member pin·메타데이터·상한을 검사한다. 원 cache/사용자 데이터/기존 ZIP 덮어쓰기 없다. 실패/부분 ZIP도 pathname으로 자동 삭제하지 않고 보존한다. 원 proof decoded bytes SHA를 pre/post baseline으로 고정하고 해시 읽기의 초기 길이·전체 상한을 검사한다. 신규 dependencies/Cargo.lock 변경 없음.

## 인수 조건

CLI flags incomplete/잘못된 SHA/누락·불일치 artifact proof/unknown member/중간 변조/초과 합계 실패; legacy CPU-only 수용. 현재 Windows source 전체4검사와 새로운 양 artifact 기록; 두 독립 create-only ZIP 동일SHA, exact companion+CPU/notice/내장폰트 검토, 외부receipt/공급도구 포함으로 오인하지 않는 README. 현재 target notice 도달성과 전체lock 누락을 분리하며 release_ready=false를 유지한다.

## 검증과 handoff

실제 same-input packaging을 root와 독립 담당이 검증했다. independent binary rebuild·cleanOS/minOS/MSVC/서명/legal complete 또는 P0 완료를 주장하지 않는다. 같은file동시수정0, root공유계약. 기능 개발 이후 Git/remote CI/updater 순서는 유지한다.

root 인수: draft9회귀 PASS 후 legacy-records companion 거절과 decoded-byte SHA2회귀를 추가해11PASS. 독립 QA에서 proof hash baseline 교체, 기존 hash growth, failed-output foreign replacement 삭제를 확인하고 실제 RED14tests/4assertions 실패를 보존했다. 원 decoded SHA baseline·stream 초기길이/최대276MiB·failed ZIP 자동삭제0으로 수정한 GREEN14tests PASS. notice12/font11 기존 회귀도 통과했다. 이는 실제 ZIP 인수를 대체하지 않는다.


최종 실제 인수: `windows_candidate_package.py`의 동일 immutable 입력으로 create-only ZIP 두개 생성 exit0; 각각 SHA `1ed2a3ef400cb8f2017e31f6c8804ce5e7300c5ae087a8034db2a363c0255c61`, 13,364,856B·864entries. `windows_candidate_package_review.py --first …/a.zip --second …/b.zip --final-proof …/result.json` exit0. 현재 source570 hash map·추가 scripts/docs/notice/whole-proof pins, GUI f3bae801…/7,917,568B와 CLI efcef0b0…/651,776B·PE GUI/Console·전체member 및 metadata를 root/독립 대조 PASS. 기존 내장font exact payload/notice verifier도 현재 CPU/ZIP/whole proof에서 exit0이다. 전체lock inventory526과 누락43, release_ready=false를 보존한다. 새로운 crate/lock 변경 없으며 CLI 크기 증가는 이전 개발 CLI 대비74,752B이다.

[원 입력·root 검토](../validation/evidence/windows-current-companion-two-zips-20261006-continue/root-review.json), [독립 검토](../validation/evidence/windows-current-companion-two-zips-20261006-continue/independent-review.json), [현재 내장폰트](../validation/evidence/windows-current-companion-font-20261006-continue/result.json). 최초 상대 script invocation의 upper/lower 프로젝트 alias ROOT 검사는 ValueError로 ZIP 생성 전 거절됐고, absolute lower script argv로 두 정상 생성을 수행했다. 실패를 성공으로 치환하거나 원 ZIP을 지우지 않았다. 실제 실행/설치·upgrade·물리 IME/reader·legal 공급은 후속이다.
