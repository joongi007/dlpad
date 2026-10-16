# S00: GPU exact revision notice 보완

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: root는 scripts/windows_candidate_package.py, docs/licenses/upstream/manifest.json 및 신규 notice6개와 이 문서; diagnostic_cache는 /tmp 수집·검토만
- 선행 작업: 최신535source 전체4명령, gl exact notice 및 missing53 후보2ZIP

## 사용자 결과

개발 후보에 gpu-alloc0.6.2, gpu-alloc-types0.3.1, gpu-descriptor0.3.2의 원 revision COPYING과 MIT 선택지 원문을 포함한다.

## 계약과 범위

Cargo/dependency/제품 실행 비용 변경0. lock inventory superset이며 실제 최종 링크 reachability/embedded 법적완비는 미확정이다. 각 cached Cargo name/version/license/repository/VCS revision/path와 원 grant bytes/SHA를 대조한다. license/MIT는 정확 세 revision의 명시 allowlist만 허용하며 일반 상대경로 허용으로 확장하지 않는다. 기존 coverage_complete=false를 유지한다. 추가 원문 총4,276B이며 압축/메타데이터 비용은 새 후보 측정으로 기록한다.

## 인수 조건

정확3mapping positive, revision/repo/path/noticeSHA 등 잘못된 mapping reject; 이전mapping 유지, 전체 source<=32/package<=64. 실제 후보2ZIP 독립 동일SHA·notice inventory 검증을 실행하고 누락 감소를 실측한다.

## 검증과 handoff

초안과 실제 로컬cache pins를 root 검토 후 통합한다. 첫 URL404/network 실패는 /tmp 원자료에 보존한다. release_ready=false를 유지한다.

root 통합 후 Python compile은 /tmp cfile 지정으로 PASS했다. 상대 uppercase cwd의 __pycache__ 쓰기 첫 read-only 오류는보존하며이후검증은lowercase absolute/소유tmp를사용했다. 실제 upstream_manifest·cachedCargo/VCS·원6파일 SHA를확인하고 positive3/negative21을 직접실행해 PASS(root-review.json)다. 정확23source/33package이고coverage_complete=false다. 첫새ZIP actual12,629,153B/854entries/missing50이며두번째재현/독립review진행중이다. source 통합은nativeRust/dependency/Cargo변경0이다.

새2ZIP은같은SHA/50누락이지만 첫독립review는license/MIT의정확nested member 경로를기존3segment전용검증기가거절했다. 첫failure raw를보존하고 root가reviewer에exactGPUpackage4개+license/MIT4segment만명시허용한다. 일반nestedpath허용/unsafe경로완화없음. scripts/windows_candidate_package_review.py를추가root독점담당하며 positive후../colon/backslash/foreignpackage reject경로를확인한다.

독립 reviewer 교정 후 두 ZIP은 SHA7ec9965e…/12,629,153B/854entries/missing50으로 일치했다. unsafe member 경로 8개를 거절했고 development-only review는 PASS, release_ready=false다. 이 작업의 원 notice3mapping은 완료했으며 전체 license/P0는 미완료다.
