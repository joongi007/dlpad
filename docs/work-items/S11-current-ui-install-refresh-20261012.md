# S11: 현재 UI 개선 설치 후보 갱신

- 상태: completed
- 분류: release
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: diagnostic_inputs — /tmp/dlpad-current-ui-package-readiness-20261012/HANDOFF.md; root — scripts/windows_candidate_package.py, scripts/test_windows_accesskit_vendor_notice.py, 패키지 통합·검증·README와 사용 안내, 이 문서
- 선행 작업: 최신 source 전체·native UI 독립 검증 완료

## 사용자 결과

현재 입력·오류·포커스 개선이 포함된 새 설치 후보로 직접 사용을 평가한다.

## 계약과 범위

기존 사용자 설치 ZIP과 invalid Git을 보존한다. 정확한 sourcegraph·법적 notice·5개 payload·manifest binding으로 새 후보를 만들고 임시 소유 폴더에서 설치한다. 서명·원격 배포·전체 notice 완료와 개발 설치 후보를 구별한다. 공유 Cargo/schema 및 실제 실행은 root가 담당한다.

## 인수 조건

새 source와 실행 파일·archive SHA가 일치하고 fresh 설치·재검증이 통과한다. 새 vendor accesskit_windows와 Chromium notice 공급 기록이 포함된다. 실제 설치 UI를 캡처하고 README의 다운로드 링크를 검증된 새 후보로 갱신한다. 이전 사용자 데이터를 덮어쓰지 않는다.

## 검증과 handoff

diagnostic_inputs에 원 pipeline 읽기 감사와 /tmp handoff를 할당했다. 현재 전체 빌드가 진행 중이므로 source export·새 manifest 생성과 패키지 실행은 보류한다. 이것은 사용자 승인 대기가 아니라 결과 의존성이다. 현재 사용자 ZIP에는 최신 수정 일부가 아직 없다.

- 읽기 감사에서 기존 patched_source map의 accesskit_windows 누락과 Chromium notice 거절·sources2 상한을 확인했다. root는 정확한 vendor 경로와 원 commit/1559B/SHA의 Chromium grant만 허용하고 accesskit_windows에 한해 정확한 MIT/Apache/Chromium 세 원문 순서를 요구하도록 통합했다. 다른 package의2개 상한은 유지했다. 실제 vendor·원문 byte 비교와 누락/외부 revision/registry·미지 버전 실패 회귀를 실행 중이며 전체 라이선스 완료를 주장하지 않는다. 이 Python 패키지 도구는 Rust whole snapshot 밖이므로 패키지 증거에서 별도 SHA를 고정한다.

- 패키지 notice 회귀90744 exit0: 실제 AccessKit vendor 경로와 세 원문 byte/SHA, 누락·외부 revision·registry/미지 version 거절 및 기존 MSVC package 회귀까지15 tests 통과. fixture ZIP은 시험용 mock 결과이며 제품 설치본으로 보고하지 않는다. 원 pipeline을 새 /tmp clone으로 보존 복사하고 tag prefix만 ui-current-final로 제한했다. 최신 whole/독립 통과에 고정한 실제 그래프91391 실행 중이다.

- 실제 graph91391 exit0: 세 원 그래프의 union258을 확인했다. accesskit_windows0.29.2는 registry 대체가 아니라 vendor/accesskit_windows/Cargo.toml의 원 path package이며 source/checksum=None이고 app graph에 포함된다. package50120을 실행 중이다. 새 사용자 ZIP의 생성·독립 archive 대조·임시 설치는 아직 완료로 주장하지 않는다.

- package50120는 아직 살아 있으며 소유 Python pipeline과 자식 package.py의 실제 process를 읽기 확인했다. 경과2분대의 파일 읽기 대기이며 terminal 성공/실패를 추정하거나 재시작하지 않는다. source snapshot648항목에 target 산출물은 없음을 확인했다. base packager SHA는 별도 root-base-packager-current-pin.json에 실행 중 관측 시점으로 기록했으며 시작 시점 pin으로 소급하지 않는다.

- package50120 exit0: selected graph258, development preview17,565,636B SHA58a451f4d4df99d3f1cb4f35a6aba559b454d33814d621e21d9304f67f3c7d64; 새 install ZIP16,515,253B SHAd96c4123e930cfe28ae7c32d1596792d071c61f826be0387ef6703b53299febb . root가 실제 archive의 accesskit_windows explicit-path-patch와3개 notices를 vendor 원 bytes/SHA로 대조했다. whole lock43 missing과 legal complete=false를 유지하며 서명·정식 release 성공으로 다루지 않는다. 독립 review9744 시작, 실제 임시 설치는 아직 실행하지 않았다.

- 독립 archive review9744와 실제 install98393 exit0. root는 default/minimum/installed/verified4개 PNG를 직접 보아 label/control/status 표시를 확인하고 root-scoped-pixel-review.json에 원 SHA와 범위를 보존했다. Win32 설치 화면의 외관을 전체 UI/UX 합격으로 다루지 않는다. 실제5개 payload/native IDs/journal의 독립 검토는 후보 준비 중이며 clean OS·서명·upgrade는 미검증이다.

- installed byte/journal 독립35288 및 실제 native ID14241 exit0. root가 두 native 원 로그·current source·모든 원 pins와 독립 canonical/native ID 대조를 다시 확인했다. 5개 payload와27개 IDs가 일치하고 설치 기록3단계 readonly/fulltext 검증을 유지한다. journal3개 PNG도 직접 확인해 총7개 설치 캡처의 좁은 가시성을 검토했다. root-independent-installed-review.json과 별도 native ID review를 보존했다. README와 windows-user-preview의 현재 링크·SHA·테스트 수를 새 검증 후보로 갱신했다. clean OS·서명·upgrade·전체 UI/UX·전체 법적 coverage 미검증은 그대로 남긴다.
