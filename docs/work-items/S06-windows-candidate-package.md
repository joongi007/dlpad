# S06: Windows 네이티브 후보의 검증 가능한 패키지

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: root는 이 문서·공유 빌드 계약·통합 및 검증, soak_recovery_audit는 /tmp/windows_candidate_package.py 및 /tmp/windows_candidate_package_plan.md 초안만 담당한다.
- 선행 작업: 고정 Windows GNU CPU native 후보와 최신 로컬 전체 검증 근거. 최종 UI/renderer 선정과 공개 릴리스 게이트는 별도 미완료다.

## 사용자 결과

검증한 실행 파일의 SHA와 구성 요소를 확인할 수 있는 비서명 개발용 portable 후보를 만든다. 실제 설치·서명·공개 배포는 별도 인수로 유지한다.

## 계약과 범위

고정 SHA와 빌드 feature를 기록하고 새 파일만 만든다. Python stdlib의 bounded streaming을 사용한다. 외부 도구·폰트·인증·사용자 데이터는 넣지 않는다. 후보 archive를 정식 installer나 릴리스 완료로 표현하지 않는다. Cargo/workspace/lock 변경은 root만 조정한다.

## 인수 조건

원 Windows PE 형식·imports·SHA·검증 snapshot과 archive의 파일 digest를 대조한다. ZIP 메타데이터를 고정하고 같은 입력의 재현성을 확인한다. 실제 라이선스 자료의 누락과 최소 OS·서명·installer·IME 등 미검증 사항을 표시하며 공개 릴리스 게이트를 우회하지 않는다.

## 검증과 handoff

현재 초안 준비 중이며 실제 패키지 생성은 미실행이다. 프로젝트 밖 데이터·전역 도구·시스템 설정 변경은 없다.

root는 `scripts/windows_candidate_package.py`를 통합하고 실제 default CPU 네 명령 근거만 허용하는 proof validator를 독립 점검했다. 긍정1·실패8case와 이전 source snapshot의 현재소스 불일치 거절을 확인했다. 실제 최신defaultCPU 전체4명령·release복원이 통과했고, root는 새ZIP 두 개와 license미완료·잘못된SHA·기존출력 보호를 실행 중이다. 표준Python3.10 호환의 제한된 lock/license inventory reader는 완전TOML/라이선스승인을 주장하지 않는다. 외부게시·서명·installer실행은 없다.

첫 실제 개발ZIP 두 개는 생성됐지만 SHA 재현성 검증이 실패했다. root는 원 archive의 전 항목을 비교해 candidate-manifest의 PE DLL 목록만 다른 것을 확인했다. 대소문자만 다른 import 이름의 정렬 key가 같아 set 순서가 남은 것이 원인이다. `(lowercase, original)`의 완전 정렬로 수정했고 별도 Python hash seed3프로세스의 실제 PE 출력 동일성을 root가 확인했다. 원 실패ZIP은 보존하고 새 두ZIP을 다시 생성 중이다.

누적 notice16MiB/entry4096 gate를 파일 읽기 전에 적용하며 JSON은 bounded chunk로 생성한다. executable256MiB 크기는 최초 hashing 전 확인한다. root는 읽기 전 예산 초과와 JSON 예산 초과의 거절을 독립 검증했다. 기존 SHA 오류·미완료license·기존출력은 실제 거절됐다. archive 전체 재현성/독립검토는 재실행 완료 후 판정한다.

수정 후 actual 새ZIP 두 개는 같은 SHA c1ea6c0c6287aa9d5f65118438a3678e0bcea261f94b70ccaa1ecf8a31524e02/12,303,027B/796항목으로 통과했다. root의 원stream executableSHA·archiveCRC·manifest대조와 별도 `scripts/windows_candidate_package_review.py`의 safe paths/no duplicate/order/1980timestamp/STORED/modes/전체streamdigest/notice대응/근거SHA 검사도 통과했다. [독립 archive 결과](../validation/evidence/windows-candidate-package-fixed-20261005-resume/independent-archive-review.json)를 따른다. 정확 original CPU7,858,176B 바이너리와 전체4명령 근거가 들어 있으며 현재 후보는 release_ready=false다.

라이선스 inventory는 lock의 superset이고 누락84건·linkedcrate/embeddednotice/SBOM완결이 남는다. 이 작업의 done은 생성전용 개발후보 archive·실패경로·같은입력 재현성의 좁은 인수다. 전체S06/R11/P0·per-userinstaller/upgrade/uninstall/media보존·signature/minimumOS/cleanmachineDLLruntime·독립소스재빌드 재현성·MSVC/공개배포의 완료가 아니다. 이후 제품소스변경 시 이 archive는 당시 snapshot으로 보존하고 새현재근거를 요구한다.

병렬license감사에서 기본CPU epaint_default_fonts의내장TTF를확인했다. 구조·digest·같은입력재현성검증PASS는유지하지만 manifest의not_bundled fonts 표기는부정확하므로이작업을다시in-progress로연다. 원ZIP은당시개발실험으로보존하고 externalfontfiles와embeddedfontpayload를구분하는정정·실제notice보완후새인수를요구한다. 기존archive를최종완성으로배포하지않는다.

2026-10-05 최신 제품의 IME 수정과 defaultCPU 전체4명령 근거로 새개발ZIP 두 개를 생성했다. 원90초 timeout의 partial archive는 보존하고240초 제한의 retry가 exit0이다. [최신 독립 검증](../validation/evidence/windows-candidate-package-current-licenses-retry-20261005-resume/independent-archive-review.json)은 PASS: 동일SHA4ce93d25d140779957425a525bb7c7fdb333dd458da245bfc7fd6efdc5eb6fd0,12,529,149B/831항목,원exe7,856,640B. manifest는 externalfontfiles와embeddedfonts를구분하며 notice추가후누락68건이다. release_ready=false와전체license/설치미완료를유지한다. 다음세션은 [인수인계](../validation/SESSION-HANDOFF-20261005.md)를따른다.
